use soroban_sdk::contracterror;

/// Errors returned by the BitGate escrow. Codes are stable: the frontend and
/// any integrator branch on them, so never reassign or remove one.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// No offer with this id.
    OfferNotFound = 1,
    /// No trade with this id.
    TradeNotFound = 2,
    /// An amount is zero, negative, or outside the offer's fill limits.
    InvalidAmount = 3,
    /// `sats_per_token` is zero.
    InvalidPrice = 4,
    /// The Bitcoin scriptPubKey is empty or longer than a standard script.
    InvalidScript = 5,
    /// `min_confirmations` is outside 1..=MAX_CONFIRMATIONS.
    InvalidConfirmations = 6,
    /// `window_secs` is outside MIN_WINDOW_SECS..=MAX_WINDOW_SECS.
    InvalidWindow = 7,
    /// `bond_bps` is above MAX_BOND_BPS.
    InvalidBond = 8,
    /// `fee_bps` is above MAX_FEE_BPS.
    InvalidFee = 9,
    /// The Bitcoin amount owed for this fill is below the dust limit, so no
    /// wallet could pay it.
    BelowDust = 10,
    /// The offer was cancelled or is fully filled.
    OfferClosed = 11,
    /// Another taker holds a reservation on this offer that has not expired.
    OfferBusy = 12,
    /// The maker cannot take their own offer.
    SelfTrade = 13,
    /// The trade was already settled or expired.
    TradeNotPending = 14,
    /// The trade's deadline has passed; it can only be expired now.
    TradeExpired = 15,
    /// The trade's deadline has not passed yet.
    TradeNotExpired = 16,
    /// `bitcoin-spv` has no checkpoint, so there is no Bitcoin tip to anchor
    /// a trade to.
    LightClientUnavailable = 17,
    /// `bitcoin-spv` rejected the proof (bad Merkle path, orphaned block,
    /// witness data in `raw_tx`, ...).
    ProofRejected = 18,
    /// The block is not yet buried under the offer's `min_confirmations`.
    NotEnoughConfirmations = 19,
    /// The block is not in the light client yet (mined in the last minutes)
    /// or is not a signet block after the checkpoint.
    BlockNotInLightClient = 20,
    /// The payment was mined at or below the Bitcoin tip recorded when the
    /// trade opened, so it cannot be a payment for this trade.
    PaymentPredatesTrade = 21,
    /// This Bitcoin transaction already settled a trade.
    TxidAlreadyUsed = 22,
    /// No output of the transaction pays the offer's scriptPubKey.
    OutputNotFound = 23,
    /// The output pays the offer's scriptPubKey less than the trade owes.
    Underpaid = 24,
    /// An arithmetic result does not fit its type.
    Overflow = 25,
}
