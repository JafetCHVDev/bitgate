import { CONFIG } from "../config";
import { reverseHex, stripWitness } from "./bitcoin";
import { merkleProof, txHex, txStatus, type EsploraMerkleProof, type EsploraStatus } from "./esplora";

/**
 * The five arguments of `bitcoin-spv.verify_transaction` minus
 * `min_confirmations`, as hex strings in internal byte order. They are also
 * the proof arguments of `BitGate.claim`.
 */
export interface SorobanProofArgs {
  block_hash: string;
  merkle_proof: string[];
  tx_index: number;
  raw_tx: string;
}

export type ProofSource = "writz-relayer" | "esplora";

export interface Proof {
  source: ProofSource;
  args: SorobanProofArgs;
  blockHeight: number;
}

/** The proof cannot be built yet; retry after `retryAfterSeconds`. */
export class ProofNotReady extends Error {
  constructor(
    readonly reason: string,
    readonly retryAfterSeconds: number,
  ) {
    super(reason);
  }
}

/**
 * Proof from the Writz relayer (`GET /spv-proof/:txid`). Its `sorobanArgs`
 * already are in the byte order the contract expects.
 */
export async function proofFromWritz(txid: string, confirmations: number): Promise<Proof> {
  const url = `${CONFIG.writzRelayer}/spv-proof/${txid}?confirmations=${confirmations}`;
  const res = await fetch(url, { signal: AbortSignal.timeout(20_000) });
  const body = await res.json().catch(() => ({}));
  if (res.ok) {
    const a = body.sorobanArgs;
    return {
      source: "writz-relayer",
      blockHeight: body.blockHeight,
      args: {
        block_hash: a.block_hash,
        merkle_proof: a.merkle_proof,
        tx_index: a.tx_index,
        raw_tx: a.raw_tx,
      },
    };
  }
  // Not ready yet, per the relayer's documented errors: wait, do not fall back.
  const notReady = ["unconfirmed", "insufficient_confirmations", "light_client_behind"];
  if (notReady.includes(body.error)) {
    const retry = Number(body.retryAfterSeconds ?? res.headers.get("Retry-After") ?? 60);
    throw new ProofNotReady(body.error, retry);
  }
  throw new Error(`Writz relayer: ${body.error ?? res.status}`);
}

/**
 * Builds the proof arguments from raw Esplora data. Esplora shows the block
 * hash and Merkle siblings in display order, so each is reversed, and its
 * transaction hex carries witness data, which is stripped.
 */
export function proofFromEsploraData(
  status: EsploraStatus,
  merkle: EsploraMerkleProof,
  hex: string,
): SorobanProofArgs {
  if (!status.confirmed || !status.block_hash) throw new ProofNotReady("unconfirmed", 60);
  return {
    block_hash: reverseHex(status.block_hash),
    merkle_proof: merkle.merkle.map(reverseHex),
    tx_index: merkle.pos,
    raw_tx: stripWitness(hex),
  };
}

/** Proof built in the browser from public Esplora APIs, no relayer involved. */
export async function proofFromEsplora(txid: string): Promise<Proof> {
  const status = await txStatus(txid);
  if (!status.confirmed) throw new ProofNotReady("unconfirmed", 60);
  const [merkle, hex] = await Promise.all([merkleProof(txid), txHex(txid)]);
  return {
    source: "esplora",
    blockHeight: merkle.block_height,
    args: proofFromEsploraData(status, merkle, hex),
  };
}

/**
 * The Writz relayer first; if it is unreachable or errors, the same proof
 * built from Esplora. A wrong proof from either source simply fails on chain:
 * neither has to be trusted.
 */
export async function getProof(txid: string, confirmations: number): Promise<Proof> {
  try {
    return await proofFromWritz(txid, confirmations);
  } catch (e) {
    if (e instanceof ProofNotReady) throw e;
    return proofFromEsplora(txid);
  }
}
