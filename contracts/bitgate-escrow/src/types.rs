use soroban_sdk::{contracttype, Address, Bytes, BytesN};

/// Protocol configuration, set at deployment.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// May change the fee, the treasury, the `bitcoin-spv` contract and itself.
    pub admin: Address,
    /// The Writz `bitcoin-spv` light client every claim is proven against.
    pub spv: Address,
    /// Receives the protocol fee on each settled trade.
    pub treasury: Address,
    /// Protocol fee on each settled trade, in basis points of the token amount.
    pub fee_bps: u32,
}

/// A maker's standing offer: tokens locked in the escrow, sold for Bitcoin
/// paid to `btc_script_pubkey` at `sats_per_token`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Offer {
    pub id: u64,
    pub maker: Address,
    /// Stellar asset contract of the token sold (USDC on testnet).
    pub token: Address,
    /// `10^decimals` of `token`: the size of one whole token in base units.
    pub unit: i128,
    /// scriptPubKey the taker must pay on Bitcoin (any script type).
    pub btc_script_pubkey: Bytes,
    /// Price: satoshis per whole token.
    pub sats_per_token: u64,
    /// Tokens still available to take, in base units.
    pub remaining: i128,
    /// Smallest fill, in base units. The last fill may be smaller.
    pub min_fill: i128,
    /// Bitcoin confirmations a payment needs before it settles a trade.
    pub min_confirmations: u32,
    /// Seconds a taker has, from `take`, to pay on Bitcoin and claim.
    pub window_secs: u64,
    /// Bond a taker locks on `take`, in basis points of the fill. Paid to the
    /// maker if the taker lets the reservation expire.
    pub bond_bps: u32,
    /// The reservation currently holding this offer, if any.
    pub active_trade: Option<u64>,
    /// Tokens sold so far, in base units.
    pub filled: i128,
    /// True once the maker cancelled it.
    pub cancelled: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    /// Reserved; waiting for the Bitcoin payment and its proof.
    Pending,
    /// Paid on Bitcoin, proven on Stellar, tokens released to the taker.
    Settled,
    /// The deadline passed without a proof; tokens went back to the offer.
    Expired,
}

/// One taker's reservation of part of an offer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub offer_id: u64,
    pub taker: Address,
    /// Tokens reserved, in base units.
    pub amount: i128,
    /// Satoshis the taker must pay to the offer's scriptPubKey.
    pub sats_due: u64,
    /// Bond the taker locked, in base units of the offer's token.
    pub bond: i128,
    /// Light client tip height when the trade opened. Only a payment mined
    /// above it can settle the trade, so an old payment to the same address
    /// cannot be reused.
    pub btc_height_floor: u32,
    pub opened_at: u64,
    /// Ledger timestamp after which the trade can no longer be claimed.
    pub deadline: u64,
    pub status: TradeStatus,
    /// Bitcoin txid that settled the trade, internal byte order.
    pub btc_txid: Option<BytesN<32>>,
    /// Height of the block that contains the payment.
    pub btc_block_height: Option<u32>,
    /// Satoshis the payment actually sent to the offer's scriptPubKey.
    pub sats_paid: Option<u64>,
    /// Protocol fee taken on settlement, in base units.
    pub fee: i128,
}

/// On-chain track record of an address, as maker and as taker.
#[contracttype]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Reputation {
    pub settled_as_maker: u32,
    pub settled_as_taker: u32,
    /// Reservations this address let expire without paying.
    pub expired_as_taker: u32,
    /// Tokens traded in settled trades, in base units, both roles summed.
    /// Mixes tokens if the address traded more than one.
    pub volume: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    OfferCount,
    TradeCount,
    Offer(u64),
    Trade(u64),
    /// A Bitcoin txid (internal byte order) that already settled a trade.
    UsedTxid(BytesN<32>),
    Reputation(Address),
}
