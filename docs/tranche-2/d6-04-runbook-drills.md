# D6-04 runbook drills

This record lists Etesia's operational runbooks and the drills that exercised them on Stellar testnet, with their receipts.

**Scope.** Testnet, pre-reset deployment (manifest `pre-2026-12-16-reset`, SHA-256 `cfa30d48…2220`); the archived XDR is the record. Guard vault `CCUHQJ3F…C6VO`; flagship `CD73M44E…XFQH`. The runbook text is [operations](https://github.com/Etesia-Research/stellar_vault/blob/master/docs/operations.md) (vault) and the keeper, indexer and monitor sections of the quant API docs (`etesia-quant/docs/api.md`).

**Class:** R, testnet receipt; S, simulation; F, local fixture network.

## Drill table

Times are UTC on 2026-11-11 unless stated. Guard receipts are lines of `etesia-vault/docs/evidence/d4-09-guard-pre-2026-12-16-reset.jsonl` (vault `602ef34`).

| # | Drill | Runbook | Evidence | Class |
| --- | --- | --- | --- | --- |
| 1 | Guardian `pause()` on the guard vault | Operations, "Price outage and pause"; "Signer roles" | `9d87935c…6f03`, ledger 5,678,997, 17:37:08 (jsonl line 19). The Paused case `d52b45ac…71b1` then failed `vault:Paused` (line 20). A4:guard opened 17:41:04 ([D6-03](d6-03-alert-evidence.md)) | R |
| 2a | Timelocked `Resume`: announce | Operations, "Role rotation and key management" (same timelock) | Admin `announce(Resume)` `801123cb…85ad`, ledger 5,675,326, 12:31; change hash `987835d9…6792`, earliest ledger 5,709,886. An early `apply_change` simulated `vault:Timelock` #18 at 5,679,006 (line 23). RPC read at about 20:00: guard paused, version 1, nonce 2, the `Resume` pending with that hash and earliest ledger | R (announce); S (early apply) |
| 2b | Timelocked `Resume`: `apply_change` | As 2a | 2026-11-13. Simulated at ledger 5,709,895: success, `config` event. Sent by `etesia-admin` (`apply_change` is permissionless at or after `earliest`): `6f4a0e63…d2aa`, ledger 5,709,899, 12:32:19 UTC, SUCCESS; `config` event version 2, change `Resume`, earliest 5,709,886, hash `987835d9…6792`. Simulated reads at ledger 5,709,903: `state()` paused false, version 2 (nonce 2, turnover 30 USDC, last_execution 5,679,002, window_ledger 5,678,203); `pending_change()` none. A4:guard resolved 12:33:09 UTC (msg 61) on the first monitor pass after the apply ([D6-03](d6-03-alert-evidence.md)). Archived: vault `103bd63`, `docs/evidence/t2-d6-04-apply-change-2026-11-13.json` (envelope, result, meta and diagnostic XDR) and `docs/evidence/t2-d6-04-apply-change-readback-2026-11-13.json` | R |
| 3 | Permissionless `unwind` while paused | Operations, "Restore liquidity without the executor" | Admin as source: `unwind([Withdraw USDC 10])` `5621980a…63b3`, ledger 5,679,002, 17:37:33 (line 21). Nonce 1 → 2; Blend supply 20.0003078 → 10.0003087 USDC | R |
| 4 | `recover_supply` while paused | Same section | `recover_supply(USDC, the rest)` `dca07db4…befa`, ledger 5,679,005, 17:37:48 (line 22). Supply → 0; spot 1,000.0003091 USDC | R |
| 5 | Reconciliation mismatch | Quant API docs, "Testnet keeper" (halt and `keeper clear`) | 2026-11-12, over `railway ssh`, quant `edcc2ae` (deployment `a5496947`): `keeper drill` ran the real `keeper.reconcile()` on the guard setup Supply `ae59d009…1d88` (line 3) with 1 USDC injected into the archived pre-read USDC spot. Halt row: journal `alerts` id 1, the flagship, 21:57:50 UTC, halt 1, reason "D6-04 drill, injected offline against a real receipt". `A13:flagship:1` opened 22:01:46 (msg 51); flagship cycle 187 (22:12:01) `halted`; `keeper clear --alert 1` at 22:12:29 → `{"cleared": 1, "halted": null}`; A13 resolved 22:17:02 (msg 52); cycle 188 (22:27:02–22:27:21) `executed`. Details [below](#reconciliation-mismatch-row-5) | R (the receipt is real; the mismatch is injected) |
| 6 | Basket exit with the backend off | Operations, "Direct USDC exit" | 2026-11-12, by a fresh Etesia test key `etesia-exit-drill` `GBQF33UD…HLLK` through `stellar contract invoke`, bounds 0.5% from the previews. Setup with the keeper on: Friendbot `create_account` `365c5c0e…1615` (ledger 5,699,983, 22:45:59), USDC `changeTrust` `96a585fb…0e08` (5,699,984), 20 USDC from `etesia-admin` `363eb5f2…fc24` (5,699,985), flagship `deposit_bounded` 20 USDC → 19.825011632929 shares `f7c43bb5…37b0` (5,699,989, 22:46:29). Keeper and indexer off (deployment `722ac145`, 22:50:43): `max_withdraw` 20.0024572 USDC before; `withdraw_bounded` 5 USDC for 4.955644038537 shares `05eb51e9…507b` (5,700,050, 22:51:34); `redeem_bounded` 14.869367594392 shares → 15.0024572 USDC `b36a848c…f79f` (5,700,052, 22:51:44); shares 0; the key holds 20.0024572 USDC. During the exit the keeper ran no cycle (cycle 189 ended 22:42:40, cycle 190 began 2026-11-13 06:49:19) and the indexer had ingested only to ledger 5,699,997, below both exit ledgers. Archived: vault `1283f16`, `docs/evidence/t2-d6-04-basket-exit-2026-11-12.json` (6 transactions) | R |
| 7 | Indexer restart and backfill | Quant API docs, "Testnet indexer and vault metrics" | 2026-11-13, re-enable deployment `95f75a79` (06:48:55 UTC). Cursor before: last ingested 5,699,997 (`A11:lag`, 2026-11-12 23:21:36); after: `next_ledger` 5,705,783, latest 5,705,782 at 06:49:21 UTC; `gaps`: no new row. Row 6 events, each once (distinct ids = rows): deposit `f7c43bb5…` 5 (mint, fees, transfer, mint, deposit; ledger 5,699,989), withdraw `05eb51e9…` 5 (mint, fees, burn, transfer, withdraw; 5,700,050), redeem `b36a848c…` 5 (mint, fees, burn, transfer, withdraw; 5,700,052). The withdraw and redeem ledgers are above 5,699,997, so the backfill ingested them. `A11:lag` resolved 06:49:18 (msg 55) | R |
| 8 | Keeper kill and restart | Quant API docs, "Testnet keeper" | D4-10 drill: with the `drop-execute-response` fault armed, cycle 70 sent `312ead0e…8a19` (nonce 53, ledger 5,677,922) and exited `ResponseLost`; after the fault was unset (deployment `5a433c28`), cycle 71 resolved it without a halt and executed `ff7bbe22…e44b` at nonce 54. No second execute at nonce 53 and no `Replay`. Archived: `docs/evidence/t2-d4-10-step2c-drill.json` (vault) | R |
| 10 | Archival and restore | Operations, "TTL and restoration" | Local node, 2026-11-11 19:57–19:58: [summary below](#archival-and-restore-drill-row-10) | F |
| 11 | Liquidity float low: alert and refill (live event) | Quant API docs, "Testnet liquidity" (refill) | The float `etesia-liquidity` `GBRMIDTC…FQEP` fell below 1,000 USDC and `A15:float` opened 2026-11-12 15:08:20 UTC (msg 45, [D6-03](d6-03-alert-evidence.md#evidence-table)). With `ETESIA_BLEND_FAUCET_URL` set in deployment `95f75a79` (2026-11-13 06:48:55 UTC), automated faucet claims paid 1,000 USDC each into the float at 06:49:39, 07:04:39 and 07:19:39; `A15:float` resolved 06:54:25; the float held 3,299.2711948 USDC at 07:19:46. Archived: vault `4e71802`, `docs/evidence/t2-liquidity-refills-2026-11-13.json` (9 transactions, ledgers 5,705,784–5,706,147) | R |
| 13 | `liquidity remove` | Quant API docs, "Testnet liquidity" (`remove`) | 2026-11-12, over `railway ssh`: the plan printed without `--yes`, then sent with `--yes`: `python -m etesia_quant.liquidity remove --data-dir /data --pair CC47HAL6FXYPVBQ6LIHKDZFVYS7XJW3N6TGRAZJSXOB2URUDTOY555UA --liquidity 100000000` (tRESERVE/USDC). `6948549cbaa4fe0ec270b2ea8c1b14783b711a80ef5f31f0b1f09cf061b0b811`, ledger 5,699,152, 21:36:44 UTC, success: 10 LP units → 10.0589250 USDC + 9.9427713 tRESERVE to `etesia-liquidity`; minimums 100,488,660 / 99,328,285 units. Sized at 10 LP to keep a 100 USDC trade within the 100 bps A15 depth threshold: the pair held 9,957.75 USDC / 9,842.76 tRESERVE, about 99.1 bps before and 99.2 bps after. Archived: [`t2-liquidity-remove-2026-11-12.json`](https://github.com/Etesia-Research/stellar_vault/blob/master/docs/evidence/t2-liquidity-remove-2026-11-12.json) (vault `47f1d62`) | R |

Rows 2b and 5–7 have step records below. Row 1 also produced the real A4 alert of [D6-03](d6-03-alert-evidence.md); row 5 its A13 alert.

## Written operations items

"Role rotation and key management" and "Deployment, signer roles and exit behavior" are in [operations](https://github.com/Etesia-Research/stellar_vault/blob/master/docs/operations.md), vault `e45ab76`:

| Item | Section | Grounding |
| --- | --- | --- |
| Exit behavior | "Direct USDC exit", "Restore liquidity without the executor", "Price outage and pause" (2026-09-18) | Unchanged |
| Deployment | "Deployment" | Constructor checks and seed (`lib.rs:118-195`); no upgrade entry point; `testnet-deploy.py vaults` and the manifest |
| Signer roles | "Signer roles" | Each role's entry points and `require_auth` lines in `governance.rs`, `execution.rs` and `lib.rs` |
| Role rotation and key management | "Role rotation and key management" | `announce`/`cancel`/`apply_change` (`governance.rs:21-115`), `ROLE_DELAY` 34,560 (`types.rs:12`); Stellar account multisig at the medium threshold, from the Stellar docs; the testnet role accounts read from Horizon |
| Local archival drill | "Local archival drill (D6-04)" | Row 10 |

Code citations are at vault `11992f9`. Read-only checks behind the text:
- Horizon testnet, 2026-11-11 19:59 UTC: each of the five role accounts (admin `GAQKH3PL…DUVY`; flagship executor `GDGCL46K…JT7N` and guardian `GCVI25VJ…FLPQ`; guard executor `GDTIM3OL…3PR6` and guardian `GCHQQLF3…DTEH`) has only its master key, weight 1, thresholds 0/0/0.
- SDF testnet RPC, 2026-11-11 20:01 UTC, simulation only: `announce --change '{"Role":["Executor","GDTIM3OL…"]}'` and `apply_change --hash 987835d9…6792` on the guard vault both parsed and failed `Error(Contract, #18)` (`Timelock`) while the `Resume` was pending; `pending_change()` returned `Resume`, earliest 5,709,886.

The operations text also records:
- A compromised guardian can cancel every announcement, its own replacement included, and hold the vault paused; exits stay open and the remedy is a new deployment (`governance.rs:60-70`; `lib.rs:49-52`).
- The monitor has no `pending_change()` check (`etesia-quant/src/etesia_quant/monitor.py`), so the guardian reviews a pending change manually; [D6-05](d6-05-mainnet-readiness-checklist.md) lists it for mainnet.

## Archival and restore drill (row 10)

**Run 2026-11-11 19:57:29–19:58:54 UTC; class F.** An unmodified copy of `etesia-vault/scripts/local-node.py` ran on an isolated protocol-30 network (Core 30.0.0, RPC 30.0.0, CLI 28.1.0, one-second ledgers) with the checked-in resource profile, except `min_persistent_ttl` and `min_temporary_ttl` lowered from 120,960 and 720 to 16 ledgers.

| Step | Observed |
| --- | --- |
| Setup | Uploaded the vault WASM `ac188cbf…1b47`, which is the code hash of both testnet vaults in the manifest. Deployed the local fixture token and minted 1,000 to the root account. Live-until ledgers 108–113 at ledger 99 |
| Expiry | At ledger 115 RPC `getLedgerEntries` returned `liveUntilLedgerSeq` 0 for all four entries: vault code, fixture code, fixture instance and the persistent balance. The RPC docs say the field "may be zero if the entry is no longer live" |
| Manual restore | The three `stellar contract restore` forms in operations ("TTL and restoration"): `--wasm-hash` (×2), `--id`, and `--id --key-xdr <balance key> --durability persistent`. Each succeeded; new live-until ledgers 131–134 (the restore ledger + 16 − 1). Balance read back 1,000 |
| Simulation while archived | `simulateTransaction` of `balance()` succeeded without a `restorePreamble`. Its `transactionData` ext was `v1` with `archived_soroban_entries` [0, 1, 2]: three footprint entries marked for restore |
| Automatic restore | A sent `mint(root, 1)` restored the fixture code, instance and balance in the same transaction (live-until 190, last modified 175); balance 1,001. The vault code entry was not in that footprint and stayed archived |

The persistent entry drilled is a fixture balance keyed by a bare `Address`. The vault's own instance and balance entries are extended to 518,400 ledgers on every vault touch (`etesia-vault/contracts/vault/src/storage.rs:4-5`).

## Drill records

### Backend off (rows 6 and 7)

- Dead-man check (A14): healthchecks.io check `etesia-stellar-keeper-deadman`, period 15 minutes, grace 45 minutes, HealthchecksBot in the monitor's Telegram chat ([D6-03](d6-03-alert-evidence.md#evidence-table)); the service pings it through `HEALTHCHECKS_PING_URL`.
- Before disabling, at 22:46 UTC (2026-11-12): 0 unresolved keeper attempts; last heartbeat cycle 189, 22:42:22–22:42:40 UTC, `executed`; indexer `cursor` next ledger 5,699,938, latest 5,699,937; `gaps` 0.
- `railway variables --set ETESIA_KEEPER_ENABLED=0 --set ETESIA_INDEXER_ENABLED=0`: deployment `722ac145`, 2026-11-12 22:50:43 UTC, SUCCESS. The monitor stayed on.
- Alerts opened: `A11:lag` 2026-11-12 23:21:36 (msg 53); `A1:flagship` 23:31:47 (msg 54); A14 DOWN 2026-11-13 00:26 UTC.
- The window ran from `722ac145` (2026-11-12 22:50:43 UTC) to `95f75a79` (2026-11-13 06:48:55 UTC), about 8 h, with no flagship keeper cycle between cycles 189 and 190.

### Re-enable, indexer restart and backfill (row 7)

- `railway variables --set ETESIA_KEEPER_ENABLED=1 --set ETESIA_INDEXER_ENABLED=1 --set ETESIA_BLEND_FAUCET_URL=…`: deployment `95f75a79`, 2026-11-13 06:48:55 UTC, SUCCESS.
- First keeper cycle: 190, 06:49:19–06:50:50 UTC, `executed` (skip: XLM->USDC inadmissible, shortfall 2570 bps). Indexer `cursor` before: last ingested 5,699,997; after: next ledger 5,705,783, latest 5,705,782 at 06:49:21 UTC.
- Indexer database `/data/indexer/pre-2026-12-16-reset.sqlite3`: 5, 5 and 5 `events` rows for the row 6 deposit, withdraw and redeem, distinct ids equal to rows; no new `gaps` row.
- Alerts resolved: `A11:lag` 06:49:18 (msg 55); `A1:flagship` 06:54:25 (msg 56); A14 UP 06:54 UTC.

### Reconciliation mismatch (row 5)

**Drill command.** Quant `edcc2ae` "Add the D6-04 reconciliation-mismatch drill command for the flagship keeper.", pushed 2026-11-12 21:55:51 UTC; Railway deployment `a5496947`, created 21:55:52 UTC, SUCCESS. `python -m etesia_quant.keeper drill --data-dir /data --receipt deploy/testnet/d4-09-guard-setup-supply.jsonl [--yes]`: the receipt file is the vault archive's jsonl line 3, shipped byte for byte in the image; `keeper.reconcile()` takes the plan, pre- and post-holdings and decoded events (`etesia-quant/src/etesia_quant/keeper.py:287-364`); a control run on the archived holdings precedes the injected one (1 USDC, 10,000,000 units, added to the archived pre-read USDC spot); the row names the flagship; the command refuses while the flagship has an uncleared halt. A halt stops only the keeper of the vault named in its row (`keeper.py:173-175, 634`), and the monitor opens A13 per vault (`A13:<vault>:<id>`, `monitor.py:253-263`), so the flagship keeper halts until `keeper clear`.

**Run, 2026-11-12, over `railway ssh`:**
- A dry run without `--yes` (about 21:57 UTC) printed `will_write` (network epoch `pre-2026-12-16-reset`, vault `CD73M44E…XFQH`, halt 1, reason "D6-04 drill, injected offline against a real receipt: reconciliation ae59d009…1d88: CAQCFVLO…RCJU spot residual -10000000. The receipt is the guard vault's (CCUHQJ3F…C6VO); this row names the flagship vault (CD73M44E…XFQH). Injected pre-holdings [...]; pool reads from the receipt meta") and "nothing written"; then `--yes` at 21:57:48–21:57:51 UTC.
- Output `{"alert": 1, "clear": "python -m etesia_quant.keeper clear --data-dir /data --alert 1"}`; journal `alerts` row id 1, vault flagship, at 2026-11-12 21:57:50 UTC (1794520670), halt 1, cleared 0.
- `monitor_alerts` id 24, key `A13:flagship:1`, opened 22:01:46 UTC, message id 51.
- Cycle 186 (21:56:41–21:57:00, the boot cycle of `a5496947`) executed before the row; cycle 187 at 22:12:01 logged `halted`, skips ["halted: D6-04 drill, injected offline against a real receipt: ..."].
- `keeper clear --data-dir /data --alert 1` at 22:12:27–22:12:29 UTC → `{"cleared": 1, "halted": null}`; the row's `cleared` = 2026-11-12 22:12:29. A13 resolved 22:17:02 UTC, message id 52. Cycle 188 (22:27:02–22:27:21) `executed`. The flagship was halted from 21:57:50 to 22:12:29; one cycle (187) was skipped as halted.

### At or after ledger 5,709,886: `apply_change` (row 2b)

Run 2026-11-13.
- At about 08:37 UTC (ledger about 5,707,080) a simulation failed `Error(Contract, #18)` (`Timelock`).
- Simulated `apply_change --hash 987835d96b7b1571e169a1a241217a584c84abcb8e86cbdbd63f0fc1b17f6792` on the guard vault at ledger 5,709,895: success, `config` event.
- Sent by `etesia-admin`: `6f4a0e63ce187c5c1c6c67d97893d5817c7ff88db30af566d0e58e4aff47d2aa`, ledger 5,709,899, 2026-11-13 12:32:19 UTC, SUCCESS; envelope, result, meta and diagnostic XDR via `record-evidence.py --testnet`; `config` event version 2, change `Resume`, earliest 5,709,886, hash `987835d96b7b1571e169a1a241217a584c84abcb8e86cbdbd63f0fc1b17f6792`.
- Simulated reads at ledger 5,709,903: `state()` paused false, version 2 (nonce 2, turnover 300000000 = 30 USDC, last_execution 5,679,002, window_ledger 5,678,203); `pending_change()` null.
- `monitor_alerts` id 4 (A4:guard) resolved 12:33:09 UTC, resolve message id 61, on the first pass after the apply (50 s after the ledger close).
- Archived in vault `103bd63`: `docs/evidence/t2-d6-04-apply-change-2026-11-13.json` and `docs/evidence/t2-d6-04-apply-change-readback-2026-11-13.json`, beside the D4-09 jsonl.
