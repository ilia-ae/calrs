# Test fixtures

Recorded answers (read-only GETs). See ilia-booking docs/ton.md.

- `toncenter/mainnet-masterchainInfo.json`: GET `https://toncenter.com/api/v3/masterchainInfo` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/testnet-masterchainInfo.json`: GET `https://testnet.toncenter.com/api/v3/masterchainInfo` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/mainnet-accountStates-active.json`: GET `https://toncenter.com/api/v3/accountStates?address=0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8&include_boc=false` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/testnet-accountStates-uninit.json`: GET `https://testnet.toncenter.com/api/v3/accountStates?address=0:5C1A3E0F9B7D24C68E01F3A5B79D2C4E6F8A0B1C3D5E7F9A2B4C6D8E0F1A3B5C&include_boc=false` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/mainnet-transactions-tf.json`: GET `https://toncenter.com/api/v3/transactions?account=0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8&start_lt=114434985000005&end_lt=115518197000030&sort=asc&limit=20` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/mainnet-latest-tf.json`: GET `https://toncenter.com/api/v3/transactions?account=0:83DFD552E63729B472FCBCC8C45EBCC6691702558B68EC7527E1BA403A0F31A8&sort=desc&limit=1` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/testnet-transactions-page1.json`: GET `https://testnet.toncenter.com/api/v3/transactions?account=0:CCA3DC4922DEAA32160983996E8482B5F179112552943DE3A0AC0D0E449A1381&start_lt=1&sort=asc&limit=100` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `toncenter/testnet-transactions-page2.json`: GET `https://testnet.toncenter.com/api/v3/transactions?account=0:CCA3DC4922DEAA32160983996E8482B5F179112552943DE3A0AC0D0E449A1381&start_lt=36122294000002&sort=asc&limit=100` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `rates/coingecko.json`: GET `https://api.coingecko.com/api/v3/simple/price?ids=the-open-network&vs_currencies=usd&include_last_updated_at=true` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `rates/binance-GRAMUSDT.json`: GET `https://api.binance.com/api/v3/ticker/24hr?symbol=GRAMUSDT` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)
- `rates/binance-TONUSDT-break.json`: GET `https://api.binance.com/api/v3/ticker/24hr?symbol=TONUSDT` (2026-11-09, recorded by scripts/ton-record-fixtures.sh)

Written by hand (not recorded):

- `toncenter/error-429.json`: the body toncenter answered with HTTP 429 on the third rapid
  request of a live session on 2026-11-09 (copied from that answer; a 429 is not provoked on
  purpose).

Derived cases (numeric `finality`, missing `mc_block_seqno`, `emulated`, external in, foreign
destination, `bounced`, encrypted comment, a jetton notify with a `B-` text, truncated JSON,
spam pages) are built at test time by `ton::decide::tests` from the recorded
`mainnet-transactions-tf.json`, so no edited JSON can drift from the real shape. The testnet
`accountStates` of a never-used account answers with an empty `accounts` list.
