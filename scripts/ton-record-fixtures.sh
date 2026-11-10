#!/bin/sh
# ilia fork: record the read-only toncenter v3 and exchange-rate answers that
# the TON payment tests parse (src/ton/, ilia-booking docs/ton.md).
#
# Only GET requests to public read endpoints. Nothing is ever sent to the
# chain, and no wallet key exists anywhere in this project. Requests are
# spaced (toncenter allows 1 request/s without a key and answers 429 after
# a burst). Optional keys come from the environment and are sent as headers
# only, never in a URL: TONCENTER_KEY (mainnet), TONCENTER_TESTNET_KEY.
#
# Usage:
#   scripts/ton-record-fixtures.sh              record the fixed set below
#   scripts/ton-record-fixtures.sh --tx NETWORK ACCOUNT LT NAME
#                                               record one transaction (by its
#                                               account and lt) as NAME.json,
#                                               e.g. after the owner sent a
#                                               testnet payment by hand
#
# Every file written is listed with its URL and the UTC date in
# tests/fixtures/README.md (appended; edit the description by hand).
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT="$ROOT/tests/fixtures"
README="$OUT/README.md"
PAUSE=${TON_RECORD_PAUSE:-3}

MAINNET=https://toncenter.com/api/v3
TESTNET=https://testnet.toncenter.com/api/v3
COINGECKO=https://api.coingecko.com/api/v3
BINANCE=https://api.binance.com/api/v3

# TON Foundation (public, busy): text comments, jetton notifies, aborted
# no_gas and bounce nofunds transfers, recorded over a fixed lt range so a
# re-run gives the same transactions.
TF=0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8
TF_FROM=114434985000005
TF_TO=115518197000030
# A testnet wallet with 100+ transactions (external-in transfers and jetton
# excesses), for a full page and the page boundary.
TN_BUSY=0:CCA3DC4922DEAA32160983996E8482B5F179112552943DE3A0AC0D0E449A1381
# A random account id nobody owns: never deployed.
TN_UNINIT=0:5C1A3E0F9B7D24C68E01F3A5B79D2C4E6F8A0B1C3D5E7F9A2B4C6D8E0F1A3B5C

today=$(date -u +%Y-%m-%d)

note() { # file url
    printf -- '- `%s`: GET `%s` (%s, recorded by scripts/ton-record-fixtures.sh)\n' \
        "$1" "$2" "$today" >>"$README"
}

get() { # out-path url [key-header]
    out=$1 url=$2 key=${3:-}
    tmp=$(mktemp)
    if [ -n "$key" ]; then
        code=$(curl -sS -m 30 -o "$tmp" -w '%{http_code}' -H 'Accept: application/json' -H "$key" "$url")
    else
        code=$(curl -sS -m 30 -o "$tmp" -w '%{http_code}' -H 'Accept: application/json' "$url")
    fi
    if [ "$code" != 200 ]; then
        echo "HTTP $code for $url" >&2
        cat "$tmp" >&2
        rm -f "$tmp"
        exit 1
    fi
    mv "$tmp" "$OUT/$out"
    chmod 644 "$OUT/$out"
    note "$out" "$url"
    echo "recorded $out"
    sleep "$PAUSE"
}

key_for() { # network
    if [ "$1" = mainnet ] && [ -n "${TONCENTER_KEY:-}" ]; then
        echo "X-Api-Key: $TONCENTER_KEY"
    elif [ "$1" = testnet ] && [ -n "${TONCENTER_TESTNET_KEY:-}" ]; then
        echo "X-Api-Key: $TONCENTER_TESTNET_KEY"
    fi
}

base_for() { # network
    if [ "$1" = mainnet ]; then echo "$MAINNET"; else echo "$TESTNET"; fi
}

mkdir -p "$OUT/toncenter" "$OUT/rates"
[ -f "$README" ] || printf '# Test fixtures\n\nRecorded answers (read-only GETs). See ilia-booking docs/ton.md.\n\n' >"$README"

if [ "${1:-}" = --tx ]; then
    [ $# -eq 5 ] || { echo "usage: $0 --tx NETWORK ACCOUNT LT NAME" >&2; exit 2; }
    net=$2 acct=$3 lt=$4 name=$5
    case $net in mainnet|testnet) ;; *) echo "network: mainnet|testnet" >&2; exit 2 ;; esac
    get "toncenter/$name.json" \
        "$(base_for "$net")/transactions?account=$acct&start_lt=$lt&end_lt=$lt&limit=1" \
        "$(key_for "$net")"
    exit 0
fi

mk=$(key_for mainnet)
tk=$(key_for testnet)

get toncenter/mainnet-masterchainInfo.json "$MAINNET/masterchainInfo" "$mk"
get toncenter/testnet-masterchainInfo.json "$TESTNET/masterchainInfo" "$tk"
get toncenter/mainnet-accountStates-active.json \
    "$MAINNET/accountStates?address=$TF&include_boc=false" "$mk"
get toncenter/testnet-accountStates-uninit.json \
    "$TESTNET/accountStates?address=$TN_UNINIT&include_boc=false" "$tk"
get toncenter/mainnet-transactions-tf.json \
    "$MAINNET/transactions?account=$TF&start_lt=$TF_FROM&end_lt=$TF_TO&sort=asc&limit=20" "$mk"
get toncenter/mainnet-latest-tf.json \
    "$MAINNET/transactions?account=$TF&sort=desc&limit=1" "$mk"

# The busy testnet wallet: its oldest 100 transactions after a fixed lt, then
# the next page from the last one + 1.
get toncenter/testnet-transactions-page1.json \
    "$TESTNET/transactions?account=$TN_BUSY&start_lt=1&sort=asc&limit=100" "$tk"
last=$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["transactions"][-1]["lt"])' \
    "$OUT/toncenter/testnet-transactions-page1.json")
get toncenter/testnet-transactions-page2.json \
    "$TESTNET/transactions?account=$TN_BUSY&start_lt=$((last + 1))&sort=asc&limit=100" "$tk"

cg_key=
[ -n "${COINGECKO_KEY:-}" ] && cg_key="x-cg-demo-api-key: $COINGECKO_KEY"
get rates/coingecko.json \
    "$COINGECKO/simple/price?ids=the-open-network&vs_currencies=usd&include_last_updated_at=true" "$cg_key"
get rates/binance-GRAMUSDT.json "$BINANCE/ticker/24hr?symbol=GRAMUSDT"
# The pair frozen since 2026-10-06 (status BREAK): the freshness rule must refuse it.
get rates/binance-TONUSDT-break.json "$BINANCE/ticker/24hr?symbol=TONUSDT"
