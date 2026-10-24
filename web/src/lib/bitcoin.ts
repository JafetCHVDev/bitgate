import { bech32, bech32m } from "bech32";

/** Bitcoin helpers that run in the browser with no Node polyfills. */

export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0 || /[^0-9a-f]/i.test(hex)) throw new Error("invalid hex");
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(2 * i, 2 * i + 2), 16);
  return out;
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

/**
 * Reverses the byte order of a hex string. Explorers show txids and block
 * hashes reversed ("display order"); bitcoin-spv works in internal order.
 */
export function reverseHex(hex: string): string {
  return bytesToHex(hexToBytes(hex).reverse());
}

/** Human-readable part of signet (and testnet) segwit addresses. */
const SIGNET_HRP = "tb";

/**
 * scriptPubKey (hex) of a signet segwit address: P2WPKH and P2WSH
 * (`tb1q...`, bech32) or P2TR (`tb1p...`, bech32m). Throws on anything else.
 */
export function addressToScript(address: string): string {
  const lower = address.trim().toLowerCase();
  if (!lower.startsWith(`${SIGNET_HRP}1`)) {
    throw new Error("Use a signet address starting with tb1q or tb1p");
  }
  let decoded: { prefix: string; words: number[] };
  let isBech32m = false;
  try {
    decoded = bech32.decode(lower);
  } catch {
    decoded = bech32m.decode(lower);
    isBech32m = true;
  }
  const version = decoded.words[0];
  const program = Uint8Array.from(bech32.fromWords(decoded.words.slice(1)));
  if (version === 0) {
    if (isBech32m) throw new Error("Witness v0 addresses use bech32, not bech32m");
    if (program.length !== 20 && program.length !== 32) throw new Error("Bad v0 program length");
  } else {
    if (!isBech32m) throw new Error("Witness v1+ addresses use bech32m");
    if (version > 16 || program.length < 2 || program.length > 40) throw new Error("Bad witness program");
  }
  const opcode = version === 0 ? 0x00 : 0x50 + version;
  return bytesToHex(Uint8Array.from([opcode, program.length, ...program]));
}

/** Signet address of a segwit scriptPubKey, or null for other script types. */
export function scriptToAddress(scriptHex: string): string | null {
  const s = hexToBytes(scriptHex);
  if (s.length < 4 || s[1] !== s.length - 2) return null;
  const op = s[0];
  const program = Array.from(s.slice(2));
  if (op === 0x00 && (program.length === 20 || program.length === 32)) {
    return bech32.encode(SIGNET_HRP, [0, ...bech32.toWords(program)]);
  }
  if (op >= 0x51 && op <= 0x60) {
    return bech32m.encode(SIGNET_HRP, [op - 0x50, ...bech32m.toWords(program)]);
  }
  return null;
}

class Reader {
  pos = 0;
  constructor(private readonly b: Uint8Array) {}
  take(n: number): Uint8Array {
    if (this.pos + n > this.b.length) throw new Error("truncated transaction");
    const out = this.b.slice(this.pos, this.pos + n);
    this.pos += n;
    return out;
  }
  varint(): number {
    const first = this.take(1)[0];
    if (first < 0xfd) return first;
    const size = first === 0xfd ? 2 : first === 0xfe ? 4 : 8;
    const bytes = this.take(size);
    let value = 0;
    for (let i = size - 1; i >= 0; i--) value = value * 256 + bytes[i];
    return value;
  }
}

/**
 * The legacy (non-witness) serialization of a raw transaction. bitcoin-spv
 * hashes `raw_tx` to get the txid, so it must not carry witness data.
 */
export function stripWitness(rawHex: string): string {
  const raw = hexToBytes(rawHex);
  if (raw.length < 6 || raw[4] !== 0x00 || raw[5] !== 0x01) return rawHex;

  const r = new Reader(raw);
  const version = r.take(4);
  r.take(2); // marker + flag
  const ioStart = r.pos;
  const inputs = r.varint();
  for (let i = 0; i < inputs; i++) {
    r.take(36);
    r.take(r.varint());
    r.take(4);
  }
  const outputs = r.varint();
  for (let i = 0; i < outputs; i++) {
    r.take(8);
    r.take(r.varint());
  }
  const ioEnd = r.pos;
  for (let i = 0; i < inputs; i++) {
    const items = r.varint();
    for (let j = 0; j < items; j++) r.take(r.varint());
  }
  const locktime = r.take(4);
  if (r.pos !== raw.length) throw new Error("trailing bytes after locktime");

  return bytesToHex(Uint8Array.from([...version, ...raw.slice(ioStart, ioEnd), ...locktime]));
}

/** Satoshi value of the first output paying exactly `scriptHex`, or null. */
export function outputValue(rawHex: string, scriptHex: string): number | null {
  const r = new Reader(hexToBytes(stripWitness(rawHex)));
  r.take(4);
  const inputs = r.varint();
  for (let i = 0; i < inputs; i++) {
    r.take(36);
    r.take(r.varint());
    r.take(4);
  }
  const outputs = r.varint();
  const want = scriptHex.toLowerCase();
  for (let i = 0; i < outputs; i++) {
    const v = r.take(8);
    let value = 0;
    for (let k = 7; k >= 0; k--) value = value * 256 + v[k];
    const spk = bytesToHex(r.take(r.varint()));
    if (spk === want) return value;
  }
  return null;
}

/** txid (display order) of a raw transaction, witness or not. */
export async function txidOf(rawHex: string): Promise<string> {
  const data = hexToBytes(stripWitness(rawHex));
  const once = await crypto.subtle.digest("SHA-256", data as BufferSource);
  const twice = await crypto.subtle.digest("SHA-256", once);
  return bytesToHex(new Uint8Array(twice).reverse());
}

/** BIP21 payment URI, understood by most Bitcoin wallets and QR scanners. */
export function bip21(address: string, sats: number, label: string): string {
  const btc = (sats / 1e8).toFixed(8).replace(/0+$/, "").replace(/\.$/, "");
  return `bitcoin:${address}?amount=${btc}&label=${encodeURIComponent(label)}`;
}
