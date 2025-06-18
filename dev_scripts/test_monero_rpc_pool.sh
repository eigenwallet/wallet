#!/usr/bin/env bash
# Smoke-test for the *restricted* (light-wallet) RPC interface of monerod.
# 2025-06-18  Author: Mohan
set -euo pipefail

NODE_URL="${1:-http://127.0.0.1:18089}"         # default port if none supplied
AUTH=""
[[ -n "${RPC_LOGIN:-}" ]] && AUTH="--user ${RPC_LOGIN}"

# Pretty printer ---------------------------------------------------------------
ok()  { printf "\e[32m[PASS]\e[0m %s\n" "$1"; }
die() { printf "\e[31m[FAIL]\e[0m %s\n" "$1"; exit 1; }

# Helpers ----------------------------------------------------------------------
json_rpc() {
  local method=$1; shift
  local params=${1:-{}}
  curl -s $AUTH -H "Content-Type: application/json" \
       -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"${method}\",\"params\":${params}}" \
       "${NODE_URL}/json_rpc"
}

legacy_get() { curl -s $AUTH "${NODE_URL}/${1}"; }

# 1. Chain height (legacy path) ------------------------------------------------
HEIGHT=$(legacy_get get_height | tr -d '\n')
[[ "$HEIGHT" =~ ^[0-9]+$ ]] || die "/get_height returned '$HEIGHT'"
ok "/get_height → $HEIGHT"

# 2. get_info ------------------------------------------------------------------
INFO=$(json_rpc get_info)
[[ "$(jq -r .result.status <<<"$INFO")" == "OK" ]] || die "get_info failed"
ok "get_info → height=$(jq -r .result.height <<<"$INFO")"

# 3. Block header at tip -------------------------------------------------------
HDR=$(json_rpc get_block_header_by_height "{\"height\":$HEIGHT}")
[[ "$(jq -r .result.block_header.height <<<"$HDR")" == "$HEIGHT" ]] \
  || die "block_header_by_height mismatch"
ok "get_block_header_by_height ($HEIGHT)"

# 4. Ten-header range ----------------------------------------------------------
START=$((HEIGHT-9))
RANGE=$(json_rpc get_block_headers_range "{\"start_height\":$START,\"end_height\":$HEIGHT}")
[[ "$(jq '.result.headers | length' <<<"$RANGE")" == 10 ]] \
  || die "header range length != 10"
ok "get_block_headers_range ($START-$HEIGHT)"

# 5. Fee estimate --------------------------------------------------------------
FEE=$(json_rpc get_fee_estimate)
[[ "$(jq -r .result.fee <<<"$FEE")" =~ ^[0-9]+$ ]] || die "fee_estimate junk"
ok "get_fee_estimate → $(jq -r .result.fee <<<"$FEE") per byte (nano-XMR)"

# 6. Output query (binary endpoint) -------------------------------------------
OUT_REQ='{"outputs":[{"amount":0,"index":1}],"get_txid":true}'
OUT_BYTES=$(curl -s $AUTH -H "Content-Type: application/json" \
                   -d "$OUT_REQ" "${NODE_URL}/get_outs.bin" | wc -c)
[[ "$OUT_BYTES" -gt 0 ]] || die "get_outs.bin empty"
ok "get_outs.bin → $OUT_BYTES bytes"

# 7. Malformed tx broadcast ----------------------------------------------------
BAD=$(json_rpc send_raw_transaction '{"tx_as_hex":"deadbeef","do_not_relay":true}')
[[ "$(jq -r .result.status <<<"$BAD")" == "Failed" ]] \
  && ok "send_raw_transaction correctly rejected garbage" \
  || die "send_raw_transaction unexpectedly accepted garbage"

printf "\n\e[32mAll restricted-RPC checks passed!\e[0m\n"

