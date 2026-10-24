#!/usr/bin/env node
// A minimal Bitcoin signet wallet for BitGate demos (P2WPKH, keys in .secrets/).
//
//   node signet-wallet.mjs new <name>                  create a wallet
//   node signet-wallet.mjs address <name>              its address
//   node signet-wallet.mjs balance <name>              confirmed and pending sats
//   node signet-wallet.mjs send <name> <addr> <sats>   pay an address
//   node signet-wallet.mjs pay-trade <name> <tradeId>  pay exactly what a BitGate trade owes
//
// Signet coins have no value. Never put a mainnet key here.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as bitcoin from "bitcoinjs-lib";
import { ECPairFactory } from "ecpair";
import * as ecc from "tiny-secp256k1";

const ECPair = ECPairFactory(ecc);
const NETWORK = bitcoin.networks.testnet; // signet shares testnet's address format
const ESPLORAS = process.env.ESPLORA
  ? [process.env.ESPLORA]
  : ["https://mempool.space/signet/api", "https://blockstream.info/signet/api"];
const ESCROW = process.env.BITGATE_ESCROW ?? "CDKVJOISYM4BWV2UQKNPBZJBPK7DX3GMK2RPZNYW2LUG7GMPNEH42TVT";
const WALLETS = join(dirname(fileURLToPath(import.meta.url)), "..", ".secrets", "wallets");
const DUST = 546n;

function load(name) {
  const file = join(WALLETS, `${name}.json`);
  if (!existsSync(file)) throw new Error(`No wallet "${name}". Create it with: new ${name}`);
  const keyPair = ECPair.fromWIF(JSON.parse(readFileSync(file, "utf8")).wif, NETWORK);
  const payment = bitcoin.payments.p2wpkh({ pubkey: keyPair.publicKey, network: NETWORK });
  return { keyPair, payment, address: payment.address };
}

/** Tries each Esplora in turn; public signet endpoints are not always up. */
async function esplora(path, init) {
  let last;
  for (const base of ESPLORAS) {
    try {
      const res = await fetch(base + path, { ...init, signal: AbortSignal.timeout(20_000) });
      const body = await res.text();
      if (res.ok) return body;
      // A rejected broadcast is an answer, not an outage: do not retry it elsewhere.
      if (init?.method === "POST" && res.status === 400) throw new Error(`Broadcast rejected: ${body}`);
      last = new Error(`${res.status} ${body}`);
    } catch (e) {
      if (String(e.message).startsWith("Broadcast rejected")) throw e;
      last = e;
    }
  }
  throw last;
}

async function get(path) {
  return JSON.parse(await esplora(path));
}

async function utxos(address) {
  return get(`/address/${address}/utxo`);
}

/** Builds, signs and broadcasts a payment of `sats` to `to`; change back to the wallet. */
export async function send(wallet, to, sats, feeRate = 2, broadcast = true, coins = null) {
  const amount = BigInt(sats);
  if (amount < DUST) throw new Error(`Below dust (${DUST} sats)`);
  const available = (coins ?? (await utxos(wallet.address))).sort((a, b) => b.value - a.value);

  const psbt = new bitcoin.Psbt({ network: NETWORK });
  let inputTotal = 0n;
  let fee = 0n;
  for (const u of available) {
    psbt.addInput({
      hash: u.txid,
      index: u.vout,
      witnessUtxo: { script: wallet.payment.output, value: BigInt(u.value) },
    });
    inputTotal += BigInt(u.value);
    // P2WPKH: ~68 vbytes per input, 31 per output, 11 overhead; two outputs.
    fee = BigInt(Math.ceil((11 + 68 * psbt.inputCount + 31 * 2) * feeRate));
    if (inputTotal >= amount + fee) break;
  }
  if (inputTotal < amount + fee) {
    throw new Error(`Not enough funds: have ${inputTotal} sats, need ${amount + fee}`);
  }

  psbt.addOutput({ address: to, value: amount });
  const change = inputTotal - amount - fee;
  if (change >= DUST) psbt.addOutput({ address: wallet.address, value: change });

  psbt.signAllInputs(wallet.keyPair);
  psbt.finalizeAllInputs();
  const tx = psbt.extractTransaction();
  const hex = tx.toHex();
  if (!broadcast) return { txid: tx.getId(), hex };

  const txid = (await esplora("/tx", { method: "POST", body: hex })).trim();
  return { txid, hex };
}

function readEscrow(fn, arg, id) {
  const out = execFileSync(
    "stellar",
    ["contract", "invoke", "--id", ESCROW, "--source-account", "bitgate-admin", "--network", "testnet", "--send=no", "--", fn, arg, String(id)],
    { encoding: "utf8" },
  );
  return JSON.parse(out);
}

async function main() {
  const [cmd, name, ...rest] = process.argv.slice(2);
  switch (cmd) {
    case "new": {
      mkdirSync(WALLETS, { recursive: true });
      const file = join(WALLETS, `${name}.json`);
      if (existsSync(file)) throw new Error(`Wallet "${name}" already exists`);
      const keyPair = ECPair.makeRandom({ network: NETWORK });
      writeFileSync(file, JSON.stringify({ wif: keyPair.toWIF() }), { mode: 0o600 });
      console.log(load(name).address);
      break;
    }
    case "address":
      console.log(load(name).address);
      break;
    case "balance": {
      const wallet = load(name);
      const coins = await utxos(wallet.address);
      const confirmed = coins.filter((u) => u.status.confirmed).reduce((s, u) => s + u.value, 0);
      const pending = coins.filter((u) => !u.status.confirmed).reduce((s, u) => s + u.value, 0);
      console.log(JSON.stringify({ address: wallet.address, confirmed, pending }));
      break;
    }
    case "send": {
      const [to, sats] = rest;
      const { txid } = await send(load(name), to, sats);
      console.log(`${txid}\nhttps://mempool.space/signet/tx/${txid}`);
      break;
    }
    case "pay-trade": {
      const trade = readEscrow("get_trade", "--trade_id", rest[0]);
      if (trade.status !== "Pending") throw new Error(`Trade ${rest[0]} is ${trade.status}`);
      const offer = readEscrow("get_offer", "--offer_id", trade.offer_id);
      const to = bitcoin.address.fromOutputScript(Buffer.from(offer.btc_script_pubkey, "hex"), NETWORK);
      const { txid } = await send(load(name), to, trade.sats_due);
      console.log(`Paid ${trade.sats_due} sats to ${to} for trade ${rest[0]}`);
      console.log(`${txid}\nhttps://mempool.space/signet/tx/${txid}`);
      break;
    }
    default:
      console.log(readFileSync(fileURLToPath(import.meta.url), "utf8").split("\n").slice(1, 10).join("\n"));
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  main().catch((e) => {
    console.error(e.message);
    process.exit(1);
  });
}
