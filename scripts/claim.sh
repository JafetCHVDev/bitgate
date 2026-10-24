#!/usr/bin/env bash
# Settles a BitGate trade from the command line.
#
#   scripts/claim.sh <trade_id> <btc_txid> [stellar_source_account]
#
# Fetches the SPV proof for <btc_txid> (display order, as on mempool.space)
# from the Writz relayer and submits it to BitGate's `claim`. Anyone may
# submit: the USDC always goes to the trade's taker.
set -euo pipefail

ESCROW="${BITGATE_ESCROW:-CDKVJOISYM4BWV2UQKNPBZJBPK7DX3GMK2RPZNYW2LUG7GMPNEH42TVT}"
RELAYER="${WRITZ_RELAYER:-https://api.testnet.writz.xyz}"
TRADE_ID="$1"
TXID="$2"
SOURCE="${3:-bitgate-admin}"

# The offer's confirmation requirement, so the relayer checks the same depth.
CONF=$(stellar contract invoke --id "$ESCROW" --source-account "$SOURCE" --network testnet --send=no -- \
  get_trade --trade_id "$TRADE_ID" | python -c "import json,sys;print(json.load(sys.stdin)['offer_id'])" |
  xargs -I{} stellar contract invoke --id "$ESCROW" --source-account "$SOURCE" --network testnet --send=no -- \
  get_offer --offer_id {} | python -c "import json,sys;print(min(json.load(sys.stdin)['min_confirmations'],20))")

PROOF=$(curl -sf "$RELAYER/spv-proof/$TXID?confirmations=$CONF") || {
  echo "Writz relayer has no proof yet:" >&2
  curl -s "$RELAYER/spv-proof/$TXID?confirmations=$CONF" >&2
  echo >&2
  exit 1
}

read -r BLOCK_HASH MERKLE TX_INDEX RAW_TX < <(printf '%s' "$PROOF" | python -c "
import json, sys
a = json.load(sys.stdin)['sorobanArgs']
print(a['block_hash'], json.dumps(a['merkle_proof'], separators=(',', ':')), a['tx_index'], a['raw_tx'])")

stellar contract invoke --id "$ESCROW" --source-account "$SOURCE" --network testnet -- \
  claim --trade_id "$TRADE_ID" --block_hash "$BLOCK_HASH" --merkle_proof "$MERKLE" \
  --tx_index "$TX_INDEX" --raw_tx "$RAW_TX"
