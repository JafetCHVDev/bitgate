import { Networks } from "@stellar/stellar-sdk";

/** Every network constant the app uses, in one place. Testnet and signet only. */
export const CONFIG = {
  networkPassphrase: Networks.TESTNET,
  rpcUrl: "https://soroban-testnet.stellar.org",
  horizonUrl: "https://horizon-testnet.stellar.org",

  /** BitGate escrow, deployed from contracts/bitgate-escrow. */
  escrowId: "CC6IZOGIOU2ME5INRC6EYCB7CDMQ5E7XKY2RT3YLRFIA3IVZACLF3XMT",
  /** Writz bitcoin-spv light client, tracking Bitcoin signet. */
  spvId: "CAHHY4S4WCIFU4LBSKBI3IP6A4QYR2COV6EACDEBXGPJFDWBJWGOMCLP",
  /** Circle testnet USDC (Stellar Asset Contract), the asset Writz Lend uses. */
  usdcId: "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
  usdcIssuer: "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5",
  usdcDecimals: 7,

  /** Writz relayer: ready-made SPV proofs for signet transactions. */
  writzRelayer: "https://api.testnet.writz.xyz",
  /** Public signet Esplora APIs: payment detection and the proof fallback. */
  esplora: ["https://mempool.space/signet/api", "https://blockstream.info/signet/api"],

  explorer: {
    stellarTx: (hash: string) => `https://stellar.expert/explorer/testnet/tx/${hash}`,
    stellarContract: (id: string) => `https://stellar.expert/explorer/testnet/contract/${id}`,
    stellarAccount: (id: string) => `https://stellar.expert/explorer/testnet/account/${id}`,
    btcTx: (txid: string) => `https://mempool.space/signet/tx/${txid}`,
    btcAddress: (address: string) => `https://mempool.space/signet/address/${address}`,
    btcBlock: (height: number) => `https://mempool.space/signet/block/${height}`,
  },

  repo: "https://github.com/JafetCHVDev/bitgate",
} as const;
