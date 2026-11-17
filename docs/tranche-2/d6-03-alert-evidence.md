# D6-03 alert evidence

This record shows what the testnet monitor checks on the two testnet vaults, where its alerts go, and every alert it delivered with its open and resolve times and Telegram message ids.

## Monitor

`etesia_quant.monitor` runs in the quant service on Railway. It went live with deploy #1 at 2026-11-11 15:24 UTC (`ETESIA_MONITOR_ENABLED=1`) and runs a pass every 300 s over both testnet vaults: flagship `CD73M44E…XFQH` and guard `CCUHQJ3F…C6VO`.

**Delivery.** Each alert is sent to Telegram once when it opens and once when it resolves: bot "EtesiaOnStellar", chat "Etesia monitor, testnet pre-2026-12-16-reset". Each incident is one row of table `monitor_alerts` in `/data/monitor/pre-2026-12-16-reset.sqlite3`: `key`, `message`, `opened_at`, `message_id`, `resolved_at`, `resolve_message_id` (Unix seconds; Telegram message ids).

**Dead-man.** A healthchecks.io check, `etesia-stellar-keeper-deadman`, runs outside the quant service (period 15 min, grace 45 min). The monitor pings it through `HEALTHCHECKS_PING_URL` only while A1 is clear, so a stopped process, a failing monitor or a stale keeper all alert. HealthchecksBot posts to the same Telegram chat; its messages name the check.

**Checks** (alert keys are `A<n>:<scope>`):

| Alert | Opens when |
| --- | --- |
| A1 | The flagship keeper's heartbeat started more than 2,700 s ago, a cycle is still running after 600 s, or no heartbeat exists. The guard has no keeper loop; A2 and A3 cover each vault's onchain liveness |
| A2 | The vault's stored target is older than 93,600 s, or no target is readable |
| A3 | The last plan execution is older than 93,600 s, or no plan has executed |
| A4 | The vault is paused |
| A5 | A `state`, `config`, `total_assets` or `holdings` simulation fails |
| A6 | A price feed is older than the vault's `max_price_age` + 300 s, or fails |
| A7 | A pinned contract's wasm hash, or the admin of Reflector, the Blend pool or the aggregator, differs from its pin |
| A8 | The aggregator's `config()` differs from the manifest |
| A9 | A vault executor holds less than 100 XLM |
| A10 | A manifest contract's instance or code entry lives fewer than 51,840 ledgers |
| A11 | The indexer is not running (`A11:not-running`), is more than 360 ledgers behind (`A11:lag`), or recorded a gap in the last 24 h (`A11:gaps`) |
| A12 | The Blend pool's USDC supply emission entry is set |
| A13 | The keeper journal holds an alert row; a halt stays open until `keeper clear` |
| A14 | healthchecks.io receives no ping within period + grace |
| A15 | Testnet liquidity account: the float holds less than 1,000 USDC (`A15:float`), a 100 USDC trade moves a pair more than 100 bps (`A15:depth:<pair>`), the day's faucet claims reached their cap (`A15:claims`), or a liquidity transaction failed in the last 24 h (`A15:failures`). Never a halt |

## Evidence table

Times are UTC on 2026-11-11 unless a date is given; open and resolve times are `monitor_alerts.opened_at` and `resolved_at`. "msg" is the Telegram message id of the OPEN or RESOLVED message.

| Alert (key) | Trigger | Opened (UTC) | Resolved (UTC) |
| --- | --- | --- | --- |
| A2 (`A2:guard`) | Real: no readable target (`vault Target` #15) until the D4-09 G1 publish `a7f3cc99…931a` (ledger 5,677,828, 15:59:43) | 15:24:50 (msg 10) | 16:03:05 (msg 14) |
| A3 (`A3:guard`) | Real: no plan executed (nonce 0) until the D4-09 setup execute `ae59d009…1d88` (ledger 5,678,203, 16:30:58) | 15:24:50 (msg 11) | 16:34:42 (msg 15) |
| A11 (`A11:lag`) | Real: at deploy #1 the indexer started behind the chain tip, backfilling from ledger 5,660,907 | 15:24:50 (msg 12) | 15:29:17 (msg 13) |
| A4 (`A4:guard`) | Real: the D6-04 guardian `pause()` `9d87935c…6f03` (ledger 5,678,997, 17:37:08), caught on the next 5-minute pass | 17:41:04 (msg 16) | 2026-11-13 12:33:09 (msg 61), on the first pass after the Resume `apply_change` `6f4a0e63…d2aa` (ledger 5,709,899, 12:32:19; [D6-04 row 2b](d6-04-runbook-drills.md#drill-table)) |
| A2 (induced) | Threshold override `target_age` | 18:03:23 (`A2:flagship` msg 17, `A2:guard` msg 20) | 18:21:54 (msgs 30, 34) |
| A3 (induced) | Threshold override `nonce_age` | `A3:guard` 18:03:23 (msg 21); `A3:flagship` 18:08:40 (msg 29) | 18:21:54 (msgs 35, 31) |
| A6 | Threshold override `feed_age` | 18:03:23 (`A6:flagship` msg 18, `A6:guard` msg 22) | 18:21:54 (msgs 32, 36) |
| A9 | Threshold override `min_xlm` | 18:03:23 (`A9:flagship` msg 19, `A9:guard` msg 23) | 18:21:54 (msgs 33, 37) |
| A10 | Threshold override `min_ttl` | 18:03:23 (`A10:ttl` msg 25) | 18:21:54 (msg 39) |
| A7 | Pin override `code` and `admin` | 18:03:23 (`A7:code` msg 24; `A7:admin:CCEBVDYM…` msg 27) | 18:21:54 (msgs 38, 41) |
| A8 | Pin override `aggregator_config` | 18:03:23 (`A8:aggregator` msg 26) | 18:21:54 (msg 40) |
| A12 | Pin override `emission` | 18:03:23 (`A12:CCEBVDYM…` msg 28) | 18:21:54 (msg 42) |
| A15 (`A15:depth:CBR76WMT…YM3V`, XLM/USDC) | Real: before the liquidity seed, selling 100 USDC moved the pair 112 bps | 2026-11-12 14:02:04 (msg 43) | 2026-11-12 15:08:20 (msg 46), after the seed (`6245231c…eaa4`, `a1f66b72…8848`, ledgers 5,694,449 and 5,694,451) put 9,900 USDC in each pair, where a 100 USDC trade moves 99.70 bps |
| A15 (`A15:depth:CC47HAL6…555UA`, tRESERVE/USDC) | Real: before the liquidity seed, 903 bps | 2026-11-12 14:02:04 (msg 44) | 2026-11-12 15:08:20 (msg 47), after the seed |
| A15 (`A15:float`) | Real: after the seed the float `GBRMIDTC…FQEP` held 289.2122698 USDC (< 1,000 USDC) | 2026-11-12 15:08:20 (msg 45) | 2026-11-13 06:54:25, after the first automated faucet claim paid 1,000 USDC into the float at 06:49:39 (`a5bc13bf…44a2`, ledger 5,705,787; 299.27 USDC before) |
| A15 (`A15:depth:CBR76WMT…YM3V`, reopened) | Real: selling 100 USDC moves XLM/USDC 107 bps; reserves 8,550.63 USDC / 80,062.99 XLM (RPC `get_reserves`, 2026-11-12 about 21:35) | 2026-11-12 18:27:09 (msg 48) | |
| A2, A3 (`A2:guard`, `A3:guard`, reopened) | Real, from the paused guard: last target 93,776 s old (> 93,600 s); nonce 2 unchanged for 93,651 s since ledger 5,679,002 (> 93,600 s) | 2026-11-12 19:38:28 (msgs 49, 50) | |
| A13 (`A13:flagship:1`) | [D6-04 reconciliation-mismatch drill](d6-04-runbook-drills.md#reconciliation-mismatch-row-5) (quant `edcc2ae`, deployment `a5496947`): a halt row for the flagship written to the keeper journal (`alerts` id 1, 21:57:50); cycle 187 `halted`; `keeper clear --alert 1` at 22:12:29; cycle 188 `executed` | 2026-11-12 22:01:46 (msg 51) | 2026-11-12 22:17:02 (msg 52) |
| A11 (`A11:lag`) | [Backend-off window](d6-04-runbook-drills.md#backend-off-rows-6-and-7): indexer stopped (`ETESIA_INDEXER_ENABLED=0`). Alert text: "last ingested ledger 5699997 is 413 ledgers behind 5700410 (> 360)". After the restart the indexer caught up to `next_ledger` 5,705,783 at 06:49:21 with no new gap | 2026-11-12 23:21:36 (msg 53) | 2026-11-13 06:49:18 (msg 55) |
| A1 (`A1:flagship`) | Backend-off window: keeper stopped (`ETESIA_KEEPER_ENABLED=0`) in deployment `722ac145` (2026-11-12 22:50:43); the last cycle before, 189, ran 22:42:22–22:42:40, `executed`. Alert text: "last keeper heartbeat 2965 s ago (> 2700 s), status executed; last attempt 2953 s ago". Re-enabled in deployment `95f75a79` (2026-11-13 06:48:55); cycle 190 (06:49:19–06:50:50) `executed` | 2026-11-12 23:31:47 (msg 54) | 2026-11-13 06:54:25 (msg 56) |
| A14 (healthchecks.io) | Backend-off window: the monitor stops pinging while A1 is open. HealthchecksBot, check `etesia-stellar-keeper-deadman`; its messages show Swiss time (UTC+1), converted here | 2026-11-13 00:26 (DOWN): "success signal did not arrive on time, grace time passed"; Total Pings 382 | 2026-11-13 06:54 (UP): "downtime lasted 6 hours, 27 minutes"; Total Pings 383 |

**Induced batch.** One override deploy opened A2, A3, A6, A9, A10, A7, A8 and A12 at once: `ETESIA_MONITOR_OVERRIDES` was set to the documented example in deployment `50838ea0` (18:03 UTC) and unset in `0f7700e4` (18:21 UTC). Threshold overrides tag their alerts `[threshold-induced: key=value]`; pin overrides tag theirs `[pin mismatch induced; no onchain change]` and change nothing onchain. The documented example:

```json
{"target_age": 60, "nonce_age": 60, "feed_age": 1, "min_xlm": 1000000, "min_ttl": 100000000,
 "code": {"CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63": "induced"},
 "admin": {"CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF": "induced"},
 "aggregator_config": {"paused": true}, "emission": "induced"}
```
