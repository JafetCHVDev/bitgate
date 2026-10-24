//! Tests run against the exact `bitcoin-spv` WASM Writz deployed on testnet
//! (fetched with `stellar contract fetch`), seeded with a real signet
//! checkpoint and fed real signet headers, and prove a real signet
//! transaction. Nothing on the Bitcoin side is mocked.

extern crate std;

use crate::fixtures::*;
use crate::{BitGateEscrow, BitGateEscrowClient, Error, Reputation, TradeStatus};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

mod spv {
    soroban_sdk::contractimport!(file = "fixtures/bitcoin_spv_testnet.wasm");
}

/// One whole USDC in base units (7 decimals).
const USDC: i128 = 10_000_000;
/// About $60,000 per BTC.
const SATS_PER_USDC: u64 = 1_667;
const WINDOW: u64 = 2 * 60 * 60;
const BOND_BPS: u32 = 200;
const FEE_BPS: u32 = 30;
/// After the timestamps of blocks 327,600 and 327,601, so the light client
/// accepts their headers.
const START_TIME: u64 = 1_792_750_000;

fn hex(s: &str) -> std::vec::Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn bytes(env: &Env, s: &str) -> Bytes {
    Bytes::from_slice(env, &hex(s))
}

fn bytes_n<const N: usize>(env: &Env, s: &str) -> BytesN<N> {
    BytesN::from_array(env, &hex(s).try_into().unwrap())
}

struct Setup<'a> {
    env: Env,
    escrow: BitGateEscrowClient<'a>,
    spv: spv::Client<'a>,
    usdc: TokenClient<'a>,
    maker: Address,
    taker: Address,
    treasury: Address,
}

struct Proof {
    block_hash: BytesN<32>,
    merkle_proof: Vec<BytesN<32>>,
    tx_index: u32,
    raw_tx: Bytes,
}

impl<'a> Setup<'a> {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(START_TIME);

        let writz_admin = Address::generate(&env);
        let spv_id = env.register(spv::WASM, (writz_admin.clone(), SIGNET_POW_LIMIT_BITS));
        let spv = spv::Client::new(&env, &spv_id);
        spv.set_checkpoint(
            &writz_admin,
            &CHECKPOINT_HEIGHT,
            &bytes_n(&env, CHECKPOINT_HASH),
            &CHECKPOINT_BITS,
            &CHECKPOINT_TIME,
            &PERIOD_START_TIME,
        );

        let usdc_admin = Address::generate(&env);
        let usdc_id = env.register_stellar_asset_contract_v2(usdc_admin).address();
        let usdc = TokenClient::new(&env, &usdc_id);

        let admin = Address::generate(&env);
        let treasury = Address::generate(&env);
        let escrow_id = env.register(
            BitGateEscrow,
            (admin, spv_id.clone(), treasury.clone(), FEE_BPS),
        );
        let escrow = BitGateEscrowClient::new(&env, &escrow_id);

        let maker = Address::generate(&env);
        let taker = Address::generate(&env);
        let minter = StellarAssetClient::new(&env, &usdc_id);
        minter.mint(&maker, &(1_000 * USDC));
        minter.mint(&taker, &(100 * USDC));

        Setup {
            env,
            escrow,
            spv,
            usdc,
            maker,
            taker,
            treasury,
        }
    }

    /// An offer of 100 USDC paid to the fixture transaction's output script.
    fn offer(&self) -> u64 {
        self.offer_with(&bytes(&self.env, PAID_SCRIPT_PUBKEY), SATS_PER_USDC, 1)
    }

    fn offer_with(&self, script: &Bytes, sats_per_token: u64, confirmations: u32) -> u64 {
        self.escrow.create_offer(
            &self.maker,
            &self.usdc.address,
            &(100 * USDC),
            script,
            &sats_per_token,
            &USDC,
            &confirmations,
            &WINDOW,
            &BOND_BPS,
        )
    }

    /// Mines blocks 327,600 (which holds the payment) and 327,601 into the
    /// light client.
    fn mine_payment_block(&self) {
        let mut headers = Vec::new(&self.env);
        for h in HEADERS {
            headers.push_back(bytes_n::<80>(&self.env, h));
        }
        assert_eq!(self.spv.submit_headers(&headers), CHECKPOINT_HEIGHT + 2);
    }

    fn proof(&self) -> Proof {
        let mut merkle_proof = Vec::new(&self.env);
        for h in MERKLE_PROOF {
            merkle_proof.push_back(bytes_n(&self.env, h));
        }
        Proof {
            block_hash: bytes_n(&self.env, BLOCK_HASH),
            merkle_proof,
            tx_index: TX_INDEX,
            raw_tx: bytes(&self.env, RAW_TX),
        }
    }

    fn claim(&self, trade_id: u64, p: &Proof) -> Result<crate::Trade, Error> {
        match self.escrow.try_claim(
            &trade_id,
            &p.block_hash,
            &p.merkle_proof,
            &p.tx_index,
            &p.raw_tx,
        ) {
            Ok(Ok(trade)) => Ok(trade),
            Err(Ok(e)) => Err(e),
            other => panic!("unexpected result: {other:?}"),
        }
    }

    fn balance(&self, who: &Address) -> i128 {
        self.usdc.balance(who)
    }
}

// ── The full trade, against the real light client ────────────────────────────

#[test]
fn settles_a_real_signet_payment() {
    let s = Setup::new();
    let offer_id = s.offer();
    assert_eq!(s.balance(&s.escrow.address), 100 * USDC);

    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    let trade = s.escrow.get_trade(&trade_id);
    assert_eq!(trade.sats_due, 16_670);
    assert_eq!(trade.bond, 10 * USDC * BOND_BPS as i128 / 10_000);
    assert_eq!(trade.btc_height_floor, CHECKPOINT_HEIGHT);
    assert_eq!(trade.deadline, START_TIME + WINDOW);
    assert_eq!(s.balance(&s.taker), 100 * USDC - trade.bond);

    s.mine_payment_block();
    let settled = s.claim(trade_id, &s.proof()).unwrap();

    let fee = 10 * USDC * FEE_BPS as i128 / 10_000;
    assert_eq!(settled.status, TradeStatus::Settled);
    assert_eq!(settled.btc_block_height, Some(CHECKPOINT_HEIGHT + 1));
    assert_eq!(settled.sats_paid, Some(PAID_SATS));
    assert_eq!(settled.fee, fee);
    assert_eq!(s.balance(&s.taker), 110 * USDC - fee);
    assert_eq!(s.balance(&s.treasury), fee);
    assert_eq!(s.balance(&s.escrow.address), 90 * USDC);

    let txid = settled.btc_txid.unwrap();
    // The txid in internal order is the explorer's txid reversed.
    let mut display = hex("1fef7b54a1819ae8cc719dec45ff5122880526fb9b8f89b303bbc260ccb718b1");
    display.reverse();
    assert_eq!(txid.to_array().to_vec(), display);
    assert_eq!(s.escrow.trade_for_txid(&txid), Some(trade_id));

    let offer = s.escrow.get_offer(&offer_id);
    assert_eq!(offer.remaining, 90 * USDC);
    assert_eq!(offer.filled, 10 * USDC);
    assert_eq!(offer.active_trade, None);

    assert_eq!(
        s.escrow.get_reputation(&s.maker),
        Reputation {
            settled_as_maker: 1,
            volume: 10 * USDC,
            ..Default::default()
        }
    );
    assert_eq!(
        s.escrow.get_reputation(&s.taker),
        Reputation {
            settled_as_taker: 1,
            volume: 10 * USDC,
            ..Default::default()
        }
    );
}

#[test]
fn claim_is_permissionless_and_pays_the_taker() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    s.mine_payment_block();

    // The proof needs no signature, so a relayer or bot can submit it.
    s.env.set_auths(&[]);
    s.claim(trade_id, &s.proof()).unwrap();
    assert!(s.balance(&s.taker) > 100 * USDC);
}

#[test]
fn a_claimed_trade_cannot_be_claimed_again() {
    let s = Setup::new();
    let trade_id = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    s.mine_payment_block();
    s.claim(trade_id, &s.proof()).unwrap();
    assert_eq!(s.claim(trade_id, &s.proof()), Err(Error::TradeNotPending));
}

// ── What the light client refuses, mapped to BitGate's own errors ────────────

#[test]
fn block_not_yet_in_the_light_client() {
    let s = Setup::new();
    let trade_id = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    assert_eq!(
        s.claim(trade_id, &s.proof()),
        Err(Error::BlockNotInLightClient)
    );
}

#[test]
fn not_enough_confirmations() {
    let s = Setup::new();
    let offer_id = s.offer_with(&bytes(&s.env, PAID_SCRIPT_PUBKEY), SATS_PER_USDC, 3);
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    s.mine_payment_block();
    // Block 327,600 has 2 confirmations; the offer asks for 3.
    assert_eq!(
        s.claim(trade_id, &s.proof()),
        Err(Error::NotEnoughConfirmations)
    );
}

#[test]
fn wrong_merkle_path_is_rejected() {
    let s = Setup::new();
    let trade_id = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    s.mine_payment_block();
    let mut p = s.proof();
    p.tx_index += 1;
    assert_eq!(s.claim(trade_id, &p), Err(Error::ProofRejected));
}

#[test]
fn tampered_transaction_is_rejected() {
    let s = Setup::new();
    let trade_id = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    s.mine_payment_block();
    // Raise the output value: the txid changes, so the Merkle proof fails.
    let mut raw = hex(RAW_TX);
    let at = raw.len() - 4 - 23 - 8;
    raw[at + 7] = 0x01;
    let mut p = s.proof();
    p.raw_tx = Bytes::from_slice(&s.env, &raw);
    assert_eq!(s.claim(trade_id, &p), Err(Error::ProofRejected));
}

// ── BitGate's own checks on top of the proof ─────────────────────────────────

#[test]
fn payment_older_than_the_trade_is_refused() {
    let s = Setup::new();
    let offer_id = s.offer();
    s.mine_payment_block();
    // The tip is now 327,601, above the payment's block.
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    assert_eq!(
        s.claim(trade_id, &s.proof()),
        Err(Error::PaymentPredatesTrade)
    );
}

#[test]
fn one_bitcoin_payment_settles_one_trade_only() {
    let s = Setup::new();
    let first = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    let second = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    s.mine_payment_block();

    s.claim(first, &s.proof()).unwrap();
    assert_eq!(s.claim(second, &s.proof()), Err(Error::TxidAlreadyUsed));
}

#[test]
fn payment_to_another_script_is_refused() {
    let s = Setup::new();
    let other = bytes(&s.env, "0014000000000000000000000000000000000000dead");
    let offer_id = s.offer_with(&other, SATS_PER_USDC, 1);
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    s.mine_payment_block();
    assert_eq!(s.claim(trade_id, &s.proof()), Err(Error::OutputNotFound));
}

#[test]
fn underpayment_is_refused() {
    let s = Setup::new();
    // 10 USDC at 20,000,000 sats per USDC owes 2 BTC; the payment is 1.999 BTC.
    let offer_id = s.offer_with(&bytes(&s.env, PAID_SCRIPT_PUBKEY), 20_000_000, 1);
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    s.mine_payment_block();
    assert_eq!(s.claim(trade_id, &s.proof()), Err(Error::Underpaid));
}

// ── Reservations, deadlines and bonds ────────────────────────────────────────

#[test]
fn a_reservation_blocks_other_takers_until_it_lapses() {
    let s = Setup::new();
    let offer_id = s.offer();
    s.escrow.take(&offer_id, &s.taker, &(10 * USDC));

    let other = Address::generate(&s.env);
    StellarAssetClient::new(&s.env, &s.usdc.address).mint(&other, &(10 * USDC));
    assert_eq!(
        s.escrow.try_take(&offer_id, &other, &(10 * USDC)),
        Err(Ok(Error::OfferBusy))
    );

    // Once the deadline passes, the next take expires the lapsed one itself.
    s.env.ledger().set_timestamp(START_TIME + WINDOW + 1);
    let maker_before = s.balance(&s.maker);
    let trade_id = s.escrow.take(&offer_id, &other, &(10 * USDC));
    assert_eq!(s.escrow.get_trade(&1).status, TradeStatus::Expired);
    assert_eq!(s.escrow.get_offer(&offer_id).active_trade, Some(trade_id));
    assert_eq!(s.escrow.get_offer(&offer_id).remaining, 90 * USDC);
    // The first taker's bond went to the maker.
    assert_eq!(s.balance(&s.maker), maker_before + 2_000_000);
}

#[test]
fn expire_returns_tokens_and_pays_the_bond_to_the_maker() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    assert_eq!(
        s.escrow.try_expire(&trade_id),
        Err(Ok(Error::TradeNotExpired))
    );

    s.env.ledger().set_timestamp(START_TIME + WINDOW + 1);
    s.mine_payment_block();
    // Too late: even a valid proof cannot settle a lapsed trade.
    assert_eq!(s.claim(trade_id, &s.proof()), Err(Error::TradeExpired));

    let maker_before = s.balance(&s.maker);
    s.env.set_auths(&[]);
    let expired = s.escrow.expire(&trade_id);
    assert_eq!(expired.status, TradeStatus::Expired);
    assert_eq!(s.balance(&s.maker), maker_before + expired.bond);
    assert_eq!(s.balance(&s.taker), 100 * USDC - expired.bond);
    assert_eq!(s.escrow.get_offer(&offer_id).remaining, 100 * USDC);
    assert_eq!(s.escrow.get_reputation(&s.taker).expired_as_taker, 1);
    assert_eq!(
        s.escrow.try_expire(&trade_id),
        Err(Ok(Error::TradeNotPending))
    );
}

#[test]
fn claim_at_the_deadline_still_settles() {
    let s = Setup::new();
    let trade_id = s.escrow.take(&s.offer(), &s.taker, &(10 * USDC));
    s.mine_payment_block();
    s.env.ledger().set_timestamp(START_TIME + WINDOW);
    s.claim(trade_id, &s.proof()).unwrap();
}

// ── Cancelling ───────────────────────────────────────────────────────────────

#[test]
fn cancel_refunds_the_unreserved_part_and_keeps_the_open_trade_valid() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));

    let maker_before = s.balance(&s.maker);
    assert_eq!(s.escrow.cancel_offer(&offer_id), 90 * USDC);
    assert_eq!(s.balance(&s.maker), maker_before + 90 * USDC);
    assert_eq!(
        s.escrow.try_cancel_offer(&offer_id),
        Err(Ok(Error::OfferClosed))
    );
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.taker, &USDC),
        Err(Ok(Error::OfferBusy))
    );

    // The taker who already reserved can still settle.
    s.mine_payment_block();
    s.claim(trade_id, &s.proof()).unwrap();
    assert_eq!(s.balance(&s.escrow.address), 0);
}

#[test]
fn a_trade_lapsing_on_a_cancelled_offer_pays_the_maker_back() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    s.escrow.cancel_offer(&offer_id);

    s.env.ledger().set_timestamp(START_TIME + WINDOW + 1);
    let maker_before = s.balance(&s.maker);
    let expired = s.escrow.expire(&trade_id);
    assert_eq!(s.balance(&s.maker), maker_before + 10 * USDC + expired.bond);
    assert_eq!(s.balance(&s.escrow.address), 0);
    assert_eq!(s.escrow.get_offer(&offer_id).remaining, 0);
}

#[test]
fn cancel_needs_the_maker() {
    let s = Setup::new();
    let offer_id = s.offer();
    s.env.set_auths(&[]);
    assert!(s.escrow.try_cancel_offer(&offer_id).is_err());
}

// ── Fills ────────────────────────────────────────────────────────────────────

#[test]
fn fills_below_min_fill_are_refused_unless_they_take_the_rest() {
    let s = Setup::new();
    let offer_id = s.offer();
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.taker, &(USDC / 2)),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.taker, &(101 * USDC)),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.taker, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

#[test]
fn the_remainder_below_min_fill_can_be_taken() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(99 * USDC + USDC / 2));
    s.mine_payment_block();
    // 99.5 USDC owes 165,867 sats; the payment is far larger.
    s.claim(trade_id, &s.proof()).unwrap();

    // 0.5 USDC is left, below the 1 USDC minimum, and is takeable as a whole.
    let other = Address::generate(&s.env);
    StellarAssetClient::new(&s.env, &s.usdc.address).mint(&other, &USDC);
    let last = s.escrow.take(&offer_id, &other, &(USDC / 2));
    assert_eq!(s.escrow.get_trade(&last).sats_due, 834);
    assert_eq!(s.escrow.get_offer(&offer_id).remaining, 0);
}

#[test]
fn a_filled_offer_is_closed() {
    let s = Setup::new();
    let offer_id = s.offer();
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(100 * USDC));
    s.mine_payment_block();
    s.claim(trade_id, &s.proof()).unwrap();
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.taker, &USDC),
        Err(Ok(Error::OfferClosed))
    );
}

#[test]
fn maker_cannot_take_their_own_offer() {
    let s = Setup::new();
    let offer_id = s.offer();
    assert_eq!(
        s.escrow.try_take(&offer_id, &s.maker, &USDC),
        Err(Ok(Error::SelfTrade))
    );
}

#[test]
fn quote_rounds_up_in_the_makers_favour() {
    let s = Setup::new();
    let offer_id = s.offer();
    // 1 base unit at 1,667 sats per 10^7 units is 0.0001667 sats: owes 1.
    assert_eq!(s.escrow.quote(&offer_id, &1), 1);
    assert_eq!(s.escrow.quote(&offer_id, &USDC), SATS_PER_USDC);
    assert_eq!(
        s.escrow.quote(&offer_id, &(3 * USDC + 1)),
        3 * SATS_PER_USDC + 1
    );
}

// ── Offer validation ─────────────────────────────────────────────────────────

#[test]
fn offer_parameters_are_validated() {
    let s = Setup::new();
    let script = bytes(&s.env, PAID_SCRIPT_PUBKEY);
    let token = s.usdc.address.clone();
    let create = |amount: i128,
                  script: &Bytes,
                  price: u64,
                  min_fill: i128,
                  conf: u32,
                  window: u64,
                  bond: u32| {
        s.escrow
            .try_create_offer(
                &s.maker, &token, &amount, script, &price, &min_fill, &conf, &window, &bond,
            )
            .err()
            .map(|e| e.unwrap())
    };

    let w = WINDOW;
    assert_eq!(
        create(0, &script, 1_667, USDC, 1, w, 0),
        Some(Error::InvalidAmount)
    );
    assert_eq!(
        create(USDC, &script, 1_667, 2 * USDC, 1, w, 0),
        Some(Error::InvalidAmount)
    );
    assert_eq!(
        create(USDC, &script, 1_667, 0, 1, w, 0),
        Some(Error::InvalidAmount)
    );
    assert_eq!(
        create(USDC, &script, 0, USDC, 1, w, 0),
        Some(Error::InvalidPrice)
    );
    let empty = Bytes::new(&s.env);
    assert_eq!(
        create(USDC, &empty, 1_667, USDC, 1, w, 0),
        Some(Error::InvalidScript)
    );
    let long = Bytes::from_slice(&s.env, &[0u8; 65]);
    assert_eq!(
        create(USDC, &long, 1_667, USDC, 1, w, 0),
        Some(Error::InvalidScript)
    );
    assert_eq!(
        create(USDC, &script, 1_667, USDC, 0, w, 0),
        Some(Error::InvalidConfirmations)
    );
    assert_eq!(
        create(USDC, &script, 1_667, USDC, 145, w, 0),
        Some(Error::InvalidConfirmations)
    );
    assert_eq!(
        create(USDC, &script, 1_667, USDC, 1, 599, 0),
        Some(Error::InvalidWindow)
    );
    let too_long = 3 * 86_400 + 1;
    assert_eq!(
        create(USDC, &script, 1_667, USDC, 1, too_long, 0),
        Some(Error::InvalidWindow)
    );
    assert_eq!(
        create(USDC, &script, 1_667, USDC, 1, w, 1_001),
        Some(Error::InvalidBond)
    );
    // 0.1 USDC at 1,667 sats per USDC is 167 sats: below dust.
    assert_eq!(
        create(USDC, &script, 1_667, USDC / 10, 1, w, 0),
        Some(Error::BelowDust)
    );
    assert_eq!(create(USDC, &script, 1_667, USDC, 1, w, 0), None);
}

#[test]
fn unknown_ids_are_reported() {
    let s = Setup::new();
    assert_eq!(s.escrow.try_get_offer(&7), Err(Ok(Error::OfferNotFound)));
    assert_eq!(s.escrow.try_get_trade(&7), Err(Ok(Error::TradeNotFound)));
    assert_eq!(
        s.escrow.try_take(&7, &s.taker, &USDC),
        Err(Ok(Error::OfferNotFound))
    );
    assert_eq!(s.escrow.try_expire(&7), Err(Ok(Error::TradeNotFound)));
    assert_eq!(s.escrow.offer_count(), 0);
    s.offer();
    s.offer();
    assert_eq!(s.escrow.offer_count(), 2);
    assert_eq!(s.escrow.trade_count(), 0);
}

#[test]
fn take_needs_a_light_client_with_a_checkpoint() {
    let env = Env::default();
    env.mock_all_auths();
    let writz_admin = Address::generate(&env);
    let spv_id = env.register(spv::WASM, (writz_admin, SIGNET_POW_LIMIT_BITS));
    let usdc_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let escrow = BitGateEscrowClient::new(
        &env,
        &env.register(
            BitGateEscrow,
            (
                Address::generate(&env),
                spv_id,
                Address::generate(&env),
                0u32,
            ),
        ),
    );
    let maker = Address::generate(&env);
    StellarAssetClient::new(&env, &usdc_id).mint(&maker, &(10 * USDC));
    let offer_id = escrow.create_offer(
        &maker,
        &usdc_id,
        &(10 * USDC),
        &bytes(&env, PAID_SCRIPT_PUBKEY),
        &SATS_PER_USDC,
        &USDC,
        &1,
        &WINDOW,
        &0,
    );
    assert_eq!(
        escrow.try_take(&offer_id, &Address::generate(&env), &USDC),
        Err(Ok(Error::LightClientUnavailable))
    );
}

// ── Admin ────────────────────────────────────────────────────────────────────

#[test]
fn fee_is_capped_and_admin_only() {
    let s = Setup::new();
    let treasury = Address::generate(&s.env);
    assert_eq!(
        s.escrow.try_set_fee(&101, &treasury),
        Err(Ok(Error::InvalidFee))
    );
    s.escrow.set_fee(&0, &treasury);
    let config = s.escrow.get_config();
    assert_eq!(config.fee_bps, 0);
    assert_eq!(config.treasury, treasury);

    s.env.set_auths(&[]);
    assert!(s.escrow.try_set_fee(&10, &treasury).is_err());
    assert!(s.escrow.try_set_spv(&treasury).is_err());
}

#[test]
fn zero_fee_and_zero_bond_trade() {
    let s = Setup::new();
    s.escrow.set_fee(&0, &s.treasury);
    let offer_id = s.escrow.create_offer(
        &s.maker,
        &s.usdc.address,
        &(10 * USDC),
        &bytes(&s.env, PAID_SCRIPT_PUBKEY),
        &SATS_PER_USDC,
        &USDC,
        &1,
        &WINDOW,
        &0,
    );
    let trade_id = s.escrow.take(&offer_id, &s.taker, &(10 * USDC));
    assert_eq!(s.balance(&s.taker), 100 * USDC);
    s.mine_payment_block();
    s.claim(trade_id, &s.proof()).unwrap();
    assert_eq!(s.balance(&s.taker), 110 * USDC);
    assert_eq!(s.balance(&s.treasury), 0);
}

#[test]
#[should_panic]
fn constructor_rejects_a_fee_above_the_cap() {
    let env = Env::default();
    env.register(
        BitGateEscrow,
        (
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            101u32,
        ),
    );
}
