import { describe, expect, it } from "vitest";
import {
  addressToScript,
  bip21,
  outputValue,
  reverseHex,
  scriptToAddress,
  stripWitness,
  txidOf,
} from "./bitcoin";
import { proofFromEsploraData } from "./proof";
import fixture from "./fixtures/esplora_1fef7b54.json";

// The P2WPKH output of signet tx 1fef7b54...b718b1 and the signet miner's
// coinbase script, with the addresses mempool.space shows for them.
const P2WPKH_SCRIPT = "0014c56f8649634e03d02308be1d660aa7959fdf643a";
const MINER_SCRIPT = "00141ebfbc64c4d28f44c7afac95df51b1281986e6de";

describe("addresses", () => {
  it("round-trips P2WPKH", () => {
    const address = scriptToAddress(P2WPKH_SCRIPT)!;
    expect(address.startsWith("tb1q")).toBe(true);
    expect(addressToScript(address)).toBe(P2WPKH_SCRIPT);
    expect(addressToScript(scriptToAddress(MINER_SCRIPT)!)).toBe(MINER_SCRIPT);
  });

  it("round-trips P2WSH and P2TR", () => {
    const p2wsh = "0020" + "ab".repeat(32);
    const p2tr = "5120" + "cd".repeat(32);
    expect(addressToScript(scriptToAddress(p2wsh)!)).toBe(p2wsh);
    const taproot = scriptToAddress(p2tr)!;
    expect(taproot.startsWith("tb1p")).toBe(true);
    expect(addressToScript(taproot)).toBe(p2tr);
  });

  it("accepts upper case and surrounding spaces", () => {
    const address = scriptToAddress(P2WPKH_SCRIPT)!;
    expect(addressToScript(`  ${address.toUpperCase()} `)).toBe(P2WPKH_SCRIPT);
  });

  it("refuses mainnet, legacy and corrupted addresses", () => {
    const address = scriptToAddress(P2WPKH_SCRIPT)!;
    expect(() => addressToScript(address.replace("tb1", "bc1"))).toThrow();
    expect(() => addressToScript("mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn")).toThrow();
    const flipped = address.slice(0, -1) + (address.endsWith("q") ? "p" : "q");
    expect(() => addressToScript(flipped)).toThrow();
  });

  it("has no address for non-segwit scripts", () => {
    expect(scriptToAddress("76a914" + "00".repeat(20) + "88ac")).toBeNull();
  });
});

describe("transactions", () => {
  it("strips witness data exactly as the Writz relayer does", () => {
    expect(fixture.hex.length).toBeGreaterThan(fixture.writzSorobanArgs.raw_tx.length);
    expect(stripWitness(fixture.hex)).toBe(fixture.writzSorobanArgs.raw_tx);
    expect(stripWitness(fixture.writzSorobanArgs.raw_tx)).toBe(fixture.writzSorobanArgs.raw_tx);
  });

  it("computes the txid with or without witness", async () => {
    expect(await txidOf(fixture.hex)).toBe(fixture.txid);
    expect(await txidOf(fixture.writzSorobanArgs.raw_tx)).toBe(fixture.txid);
  });

  it("reads the value paid to a script", () => {
    expect(outputValue(fixture.hex, P2WPKH_SCRIPT)).toBe(199_948_796);
    expect(outputValue(fixture.hex, MINER_SCRIPT)).toBeNull();
  });

  it("reverses byte order", () => {
    expect(reverseHex("0a0b0c")).toBe("0c0b0a");
    expect(reverseHex(reverseHex(fixture.txid))).toBe(fixture.txid);
  });

  it("builds a BIP21 URI", () => {
    expect(bip21("tb1qxyz", 16_670, "BitGate trade 1")).toBe(
      "bitcoin:tb1qxyz?amount=0.0001667&label=BitGate%20trade%201",
    );
    expect(bip21("tb1qxyz", 100_000_000, "x")).toBe("bitcoin:tb1qxyz?amount=1&label=x");
  });
});

describe("proof fallback", () => {
  it("builds from Esplora the same arguments the Writz relayer returns", () => {
    const args = proofFromEsploraData(fixture.status, fixture.merkleProof, fixture.hex);
    expect(args).toEqual(fixture.writzSorobanArgs);
  });

  it("refuses an unconfirmed transaction", () => {
    expect(() =>
      proofFromEsploraData({ confirmed: false }, fixture.merkleProof, fixture.hex),
    ).toThrow("unconfirmed");
  });
});
