// Real Bitcoin signet data for tests. Generated from the Writz relayer
// (GET /spv-proof/1fef7b54...b718b1?confirmations=2) and the Blockstream
// signet Esplora API. Hashes are in internal byte order.

/// Block 327,599: the checkpoint the light client is seeded with in tests.
pub const CHECKPOINT_HEIGHT: u32 = 327_599;
pub const CHECKPOINT_HASH: &str =
    "dfdb05c92747ec34926d01dfc59bb7490f907ab9698654fbbedd7ab90b000000";
pub const CHECKPOINT_BITS: u32 = 487_935_899;
pub const CHECKPOINT_TIME: u32 = 1_792_744_829;
/// Timestamp of block 326,592, the first block of the checkpoint's difficulty period.
pub const PERIOD_START_TIME: u32 = 1_792_145_465;
/// Signet proof-of-work limit (compact).
pub const SIGNET_POW_LIMIT_BITS: u32 = 0x1e03_77ae;

/// Headers of blocks 327,600 and 327,601.
pub const HEADERS: [&str; 2] = [
    "00000020dfdb05c92747ec34926d01dfc59bb7490f907ab9698654fbbedd7ab90b0000002da9441d8297ef5e4b364541bd67997eac3cc2f6401ff6f49e3012461966e3204c21db6a9b4f151dbc556108",
    "000000209e4d5092aeb95f259afd73c770b9238bee9f300237de7a026150858e0f00000045497b9d26a04827cc395ba567bab30873bc4ec583a0423780e34fdf02613d7c6523db6a9b4f151d1658f10a",
];

/// Transaction 1fef7b54...b718b1 (display order), index 1 in block 327,600.
pub const BLOCK_HASH: &str = "9e4d5092aeb95f259afd73c770b9238bee9f300237de7a026150858e0f000000";
pub const TX_INDEX: u32 = 1;
pub const RAW_TX: &str = "0200000001b537900472b560534e675775bb97d3cc4c74147887e34174e7b838cfa8bfa5d60000000000b801000001fcf9ea0b00000000160014c56f8649634e03d02308be1d660aa7959fdf643a00000000";
pub const MERKLE_PROOF: [&str; 9] = [
    "f3e0901faf36859f1dc32714e50c3cf1ec23b066d9142d8c21c9aebc3a6e9c02",
    "2f84bb5ecb1ee9a6e54b23ac4d6d46b8ba9dd4fbb529065b552afbd71924c1e9",
    "c2bbd35a908f183585a87f74df4bbdabd68b1feb09e5844737d3cd35ebe94e6a",
    "73ed56a02f6c257478e46f2113ba07642b0445e91f45deaa4e0a607b18294445",
    "d6344f1cb17bd9a437c42a5fa343d7517420a4940aef753463d3a8988fadf8ab",
    "5d99d1b9f9aab912213b8201853e7473009912bf28d37f8c498da4cfa1675a44",
    "37e4b9380cf16210ca94c025c44cd0a8233d9b3372a0e74301b8f11f30355d32",
    "d86e08c0bde919a254ce9f084f20a4f40319928c3d8de42e0ce524e1c13859f1",
    "f5e310950429577c68bbc4707fbb14d6fd3bc66da5af104f17bde777ed9dc1fe",
];
/// The transaction's only output: 199,948,796 sats to this P2WPKH scriptPubKey.
pub const PAID_SCRIPT_PUBKEY: &str = "0014c56f8649634e03d02308be1d660aa7959fdf643a";
pub const PAID_SATS: u64 = 199_948_796;
