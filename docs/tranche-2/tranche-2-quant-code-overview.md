# Tranche 2 quant code overview

This page describes the quant service that drives the Tranche 2 testnet vaults: keeper, planner, event indexer, metrics API, monitor and testnet liquidity. It describes `etesia-quant` `master` `1e8d2aa` (2026-11-13), deployed on Railway as `b9b292ed` the same day. The repository, `Etesia-Research/stellar_quant`, is private and open to reviewers on request. Paths are relative to `src/etesia_quant/` unless another path is given. The vault contracts and the receipt archive are public in [`Etesia-Research/stellar_vault`][vault] ([archive index][index]).

## Contents

1. [Position in the system](#1-position-in-the-system)
2. [Service layout on Railway](#2-service-layout-on-railway)
3. [Strategy decision and target builder](#3-strategy-decision-and-target-builder-d1-d4-04)
4. [Triggers](#4-triggers-d4-07)
5. [Keeper](#5-keeper-d4-05-d4-06)
6. [Target and Plan wire format](#6-target-and-plan-wire-format)
7. [Event indexer](#7-event-indexer-d5-04)
8. [HTTP API and metrics routes](#8-http-api-and-metrics-routes-d5-05)
9. [Monitor and alerts](#9-monitor-and-alerts-d6-03)
10. [Testnet liquidity](#10-testnet-liquidity)
11. [Configuration names](#11-configuration-names)
12. [Test suite](#12-test-suite)
13. [Scope of the code](#13-scope-of-the-code)
14. [Evidence index](#14-evidence-index)

## 1. Position in the system

```text
User wallet in the Builder page ──signs deposit_bounded / withdraw_bounded / redeem_bounded──▶ testnet vaults
        │ (browser)
Builder server: Next.js route /api/quant/[...path] (allowlisted paths; bearer key kept server-side)
        │ HTTPS, Authorization: Bearer <key>
        ▼
Quant service (FastAPI, one Railway service, one replica, one worker, persistent /data volume)
   ├─ HTTP API: Builder targets and analytics, Soroswap quotes, backtest reports, vault metrics (D5-05)
   ├─ refresh loop: daily market data, the 01:00 UTC decision inputs, the vault backtest
   ├─ keeper loop (D4): the only vault signer; publish_target and execute on the flagship vault
   ├─ indexer loop (D5-04): read-only getEvents and simulated views into SQLite
   ├─ monitor loop (D6-03): read-only checks; Telegram alerts and a healthchecks.io ping
   └─ liquidity: testnet pool top-ups and realigns from the etesia-liquidity float
      (keeper hook, POST /v1/testnet/liquidity) and an hourly Blend faucet refill
                                   │
                                   ▼
          Stellar testnet RPC: flagship and guard vaults, Reflector-backed pricing provider,
          Soroswap v3 aggregator copy and pairs, Blend pool
```

- **One vault signer.** Only the keeper signs vault calls, with the flagship executor key, and only `publish_target` and `execute`. The indexer and the monitor are keyless; their simulations use a public account as source and are never signed or sent. A separate testnet key, the `etesia-liquidity` account, signs Soroswap router `add_liquidity`, `swap_exact_tokens_for_tokens` (realigns) and operator `remove_liquidity` calls, and fresh keys derived from it sign faucet claims (section 10). It holds no vault role.
- **Users sign their own transactions.** Deposits and exits are built by the Builder page and signed in the user's wallet. The indexer reads them once they are onchain.
- **Testnet only.** `network.load` checks the manifest file against its pinned SHA-256, refuses any passphrase but testnet, and requires every vault's asset list to equal the manifest's. The public quote route reads mainnet; the market-data history reads CoinGecko and mainnet Horizon. Neither signs anything.
- **Pinned deployment.** The manifest is `deploy/testnet/pre-2026-12-16-reset.json`, SHA-256 `cfa30d48…2220`, the same file and pin as the [archive index][index]. It lists USDC (Settlement), tRESERVE (the Reserve stand-in) and XLM; the flagship `CD73M44E…XFQH` and guard `CCUHQJ3F…C6VO` vaults with their executors; the pricing provider `CC3ASLC6…X5EB`; the v3 aggregator copy `CBE32GUT…V5TH`; the Blend pool `CCEBVDYM…4HGF`; two Soroswap pairs; and the expected wasm hash of every contract.
- **The guard vault runs no keeper loop.** Its D4-09 cases are sent by `scripts/d4-09-guard-cases.py`, which signs through Stellar CLI identities (`stellar tx sign`), never reads a secret, checks each signature against the manifest address and sends nothing unless the simulation returns the expected code ([guard receipts][d409]).

## 2. Service layout on Railway

The image is built from the repository `Dockerfile`: Python 3.14.7, uv 0.12.9, `uv sync --locked --no-dev`, with the manifest under `deploy/`. Uvicorn serves `etesia_quant.api:create_app` with one worker. `.railway/railway.ts` declares one service, one replica, the `/healthz` healthcheck and the `/data` volume.

`create_app` starts each enabled loop. Each loop runs one cycle as a subprocess (`python -m etesia_quant.<module> once`), so a slow RPC call never blocks the API worker. Refresh, keeper, indexer and liquidity cycles take a non-blocking `writer.lock` on their directory (`refresh.writer_lock`), so a second instance exits instead of writing the same store. The monitor writes only its own alert table.

| Loop | Enabled by | Period | Store under `/data` | Signs |
| --- | --- | --- | --- | --- |
| HTTP API | always | — | reads all stores below | no |
| Refresh (`refresh.py`) | `ETESIA_REFRESH_ENABLED` (default on) | collection from 00:40 UTC; the vault backtest once the day's closes are complete and it is past 01:00 UTC (`refresh_due`) | `history/`, `markets/`, `market.json`, `runs/`, `latest.json`, `refresh.json` | no |
| Keeper (`keeper.py`) | `ETESIA_KEEPER_ENABLED` | one cycle, then 900 s (`CYCLE_SECONDS`) | `keeper/journal.sqlite3`, `keeper/evidence/`, `keeper/writer.lock` | yes: flagship executor |
| Indexer (`indexer.py`) | `ETESIA_INDEXER_ENABLED` | one cycle, then 300 s | `indexer/<network_epoch>.sqlite3` | no |
| Monitor (`monitor.py`) | `ETESIA_MONITOR_ENABLED` | one run, then 300 s | `monitor/<network_epoch>.sqlite3` | no |
| Liquidity refill (`liquidity.py`) | `ETESIA_LIQUIDITY_ENABLED` | one `refill` run, then 3,600 s | `liquidity/<network_epoch>.sqlite3`, `liquidity/evidence/`, `liquidity/writer.lock` | yes: `etesia-liquidity` and fresh faucet keys |

The keeper, indexer, monitor and liquidity loops are off unless their flag is `1`.

## 3. Strategy decision and target builder (D1, D4-04)

**Decision** (`strategy.py`). Once a day the model takes a decision at 01:00 UTC from daily closes up to 00:00 UTC, published by the refresh loop as an immutable, hash-named market snapshot. Parameters are versioned as `d1-v6` (`numerics.py`). The risk universe is XLM, AQUA, ETH and BTC: a trend signal on up to 731 daily closes, equal-risk-contribution weights, a 30% annual volatility budget, 40% per-asset caps (37.5% for XLM, 40% with its 2.5% buffer), a 95% gross trend cap and 2.5% USDC and XLM buffers. The decision is `valid` or `blocked` with diagnostics (for example a USDC depeg beyond ±1%, missing closes, or no eligible asset) and carries `execution_authorized: false`; the decision alone authorizes nothing.

**Testnet mapping `t2-map-v1`** (`target_builder.weights`). The testnet manifest lists USDC, tRESERVE and XLM, so the rule uses the decision's XLM exposure `n_XLM`:

- `w_XLM = clamp(floor(n_XLM × 10,000) + 250, 250, 4,000)` bps (250 is the XLM buffer);
- `R = 10,000 − w_XLM − 250`;
- `w_USDC = 250 + floor(R × yield_bps / 10,000)` and `w_tRESERVE = R − floor(R × yield_bps / 10,000)`.

The flagship uses `yield_bps` 1,000 (`FLAGSHIP_YIELD_BPS`). The rule and its 200 bps trigger band are hashed into every target's `model` field (`model_hash`).

**Target** (`target_builder.target`). A target holds `network`, `vault`, `epoch` (Unix time), `expiry` (ledger time + 86,400 s), `model`, `dataset` (the market snapshot id), `holdings` (`holdings_hash()`), `assets`, `weights`, `yield_bps`, `equity` (`total_assets()`) and `supply` (`collect_fees().supply`), the last three read by simulation when the target is built. Before anything is published, `validate` mirrors the vault's `validate_target` and `publish_target` checks:

- binding to this network and vault;
- expiry within 24 h;
- equity and supply within 100 bps of the live reads, and the same holdings hash;
- weights summing to 10,000, with Settlement ≥ 250, XLM 250–4,000, Risk ≤ 4,000 and Reward 0;
- the yield identity;
- a configured pool holding USDC when `yield_bps` > 0;
- a strictly increasing epoch.

**Scenario targets** (`triggers.scenario_target`). `ETESIA_KEEPER_SCENARIO` can name a scenario, `flat` (`n_XLM` 0) or `xlm-1500` (`n_XLM` 0.125). It runs through the same encoder, and its `model` and `dataset` hash the scenario name. While a scenario is set, decisions are computed and archived but not published.

## 4. Triggers (D4-07)

`triggers.targets` decides, in every keeper cycle, whether a target is published and a plan built (`fire`). Distances use the vault's own formula, mirrored in `planner.distance`.

| Label | Fires when |
| --- | --- |
| Scheduled | The day's 01:00 decision is computable and no earlier cycle of this vault recorded it. The decision is archived as a `d4-scheduled-decision` record |
| Signal | That new decision's weights differ from the stored target's by more than 200 bps in summed absolute weight change |
| Drift | The vault's distance to the candidate target exceeds 200 bps of equity |

The candidate target is the valid decision. On a day without one, the stored target's weights are re-published with a fresh epoch and expiry. Every evaluation (labels, distance, band, turn, scenario, archive path) is written to that cycle's heartbeat row in the journal. Under a scenario, only Drift leads to a trade.

## 5. Keeper (D4-05, D4-06)

`keeper.py` runs one cycle per subprocess for the vault named by `ETESIA_KEEPER_VAULT` (the flagship). `Keeper.__init__` refuses a key whose public address is not the manifest executor of that vault.

### 5.1 One cycle, in order (`Keeper._cycle`)

1. Resolve every unresolved journal attempt with `getTransaction` (`settle`).
2. Stop if the vault has an uncleared halt.
3. Read `state()` and `config()`. Skip on the vault cooldown, for one cycle after a `skip` outcome, and after a `Pricing` failure until a newer Reflector round exists.
4. Read the Reflector marks through the pricing provider and open the D4-05 submission window, waiting for the next round if needed (5.3).
5. Build the target (sections 3 and 4) and sign `publish_target`. A `retry` outcome republishes once.
6. Read `target()` back. If its hash is not the one just published, record a halt.
7. Read `state()`, `holdings()` and the Blend position. Set the deadline to `min(now + 120 s, target expiry, marks horizon)` and skip if under 30 s remain. Read the venue; with `ETESIA_LIQUIDITY_KEEPER=1`, run the liquidity hook (`_top_up`, section 10); then build one plan.
8. Sign `execute(plan)`. A `retry` outcome resubmits once. A plan over the transaction budget is split (5.6).
9. Write the heartbeat status (`executed`, `skipped`, `halted`, …) with its skip reasons.

At most one `publish_target` succeeds and at most one `execute` is sent per cycle, apart from a single same-cycle retry.

### 5.2 Onchain calls

| Call | Contract | Kind | Purpose |
| --- | --- | --- | --- |
| `state`, `config`, `holdings`, `target`, `total_assets`, `collect_fees`, `holdings_hash` | vault | simulated read | cooldown, limits, target binding, holdings, readback |
| `mark(asset)` | pricing provider (Reflector-backed) | simulated read | marks and their round timestamp |
| `get_pair(a, b)` | Soroswap factory | simulated read | the pair a quote routes through |
| `config` | v3 aggregator copy | simulated read | `protocol_bps` fee |
| `get_reserve`, `get_positions` | Blend pool | simulated read | bTokens and `b_rate` before and after a plan |
| `publish_target(Target)` | vault | **signed** by the executor | store the target |
| `execute(Plan)` | vault | **signed** by the executor | rebalance through the v3 copy and Blend |

RPC methods used are `getNetwork`, `getLatestLedger`, `getLedgerEntries`, `simulateTransaction`, `sendTransaction`, `getTransaction` and `getEvents`. Every envelope is simulated first and prepared with the simulated footprint and resource fee (`_envelope`). A failed simulation raises `Unsimulated` and nothing is signed; so does an authorization entry that needs any signature other than the source account's.

### 5.3 Planning, quotes and timing (D4-05)

- **Reflector window** (`routing.submit_window`). The keeper quotes only right after a Reflector round has landed, and the plan must be includable before the next round can land: the horizon is `min(300 s + 25 s, max_price_age)` against a 180 s inclusion bound (`REFLECTOR_RESOLUTION`, `MIN_PUBLICATION_DELAY`, `MAX_INCLUSION_LATENCY`). Otherwise it polls for the next round, bounded if ledgers stall.
- **Quotes** (`routing.quote`). `POST /quote` on the Soroswap API with `network=testnet`, `protocols: ["soroswap"]`, `EXACT_IN`, one part, one hop. Only HTTP 429 and 503 with a `Retry-After` are retried, and only while the deadline leaves time to execute. Any other quote error skips the cycle with its status as the reason.
- **Route translation** (`routing.translate`). A quote becomes a vault route only if it is a one-hop Soroswap router path between manifest assets for exactly the amount less the v3 copy's fee, through a pair in the vault's `route_contracts`.
- **Admissibility and minimum output** (`routing.leg`, `min_out`). A leg is admissible when the quoted output, valued at Reflector marks, is within `max_slippage_bps − 10` bps of the input's value. `minimum` is the larger of the quote's threshold and the vault's own Reflector floor plus 5 bps of headroom.
- **Sizing** (`planner._sizing`, `planner.build`, `routing.solve`). Each leg is quoted at its budget size (turnover room, leg cap, cash above the floor), then at half, then at a size solved from a constant-product fit of those two quotes. Supply leaves `reserve = ceil(limit × max_slippage_bps / 10,000)` free below the yield limit less the accrual margin (1% at the testnet's 100 bps), so that a swap sent alone does not push the supply over the limit it lowers, and Supply is planned only when it adds at least the reserve. Each leg is capped to that headroom, `leg_cap × (limit − supplied − margin) / leg_fall`, where `leg_fall` is the limit's fall when a leg-cap leg fills at the slippage floor. A lone re-entry Withdraw lands at most the margin plus the reserve below the limit, so that alone it still lowers the distance.
- **Order and checks.** A plan is Withdraw on re-entry, sells to USDC, buys from USDC, then Supply last, with at most 8 actions. `planner.checks` mirrors the vault's `execute` checks on the projected holdings:
  - action count;
  - `max_leg_bps` per leg;
  - the distance to the target must fall;
  - USDC and XLM spot floors at 2.5% of equity, planned with 2.55% headroom;
  - Risk and XLM at most 40% of equity while borrowing is off;
  - the Blend yield cap, applied strictly as in the deployed vault;
  - the turnover and loss windows.

  A plan that fails any check is not built; its reasons are recorded as skips.

### 5.4 Submission, journal and idempotency (D4-06)

The journal is SQLite at `/data/keeper/journal.sqlite3` with three tables: `attempts`, `heartbeats` and `alerts` (`Journal`).

- **Hash before send** (`submit`, `_send`, `settle`). The signed envelope and its hash are inserted before `sendTransaction`. A lost response is settled on the next run by polling `getTransaction` for that hash; if no response came, the same envelope is resent, and a resend is not a new attempt. An attempt resolves `never-included` when a ledger closes past its `maxTime`, or when its first send is rejected with an error and `getTransaction` does not find it.
- **One in flight.** A partial unique index allows one unresolved attempt per source account (`one_in_flight`), and `submit` refuses to sign while one exists. A trigger forbids changing a resolved outcome (`one_terminal`).
- **No duplicate execute.** Each `Plan` binds the vault nonce, configuration version, target hash, epoch and a deadline. With the in-flight rule, a restarted keeper resolves the earlier `execute` before building a new one, and a nonce is signed again only after its earlier envelope is resolved (`never-included` or `included-failed`), so that envelope can no longer land. The vault rejects a stale nonce with `Replay`.
- **Fault hook.** `ETESIA_KEEPER_FAULT` drops the first publish or execute response after the hash is journalled, to exercise the restart path. It is ignored off testnet.

### 5.5 Classification and outcomes

An included failure is classified from the first `error` diagnostic event of a manifest contract, named by role (`vault`, `aggregator`, `pair`, `pool`, `token`, `router`) and by the contract's error enum, for example `vault:Limit` or `aggregator:InsufficientOutput` (`classify`). It is never classified by re-simulation. `action` maps the outcome:

| Outcome or classification | Action |
| --- | --- |
| `success` | ok |
| never included, past `maxTime` (`expired`) | rebuild next cycle |
| the same classification as the previous resolved attempt of that vault and kind | **halt** |
| `send:txBAD_SEQ`, `send:txINSUFFICIENT_FEE`, `send:txTOO_LATE`, `unclassified` | rebuild |
| storage or footprint error (`host:storage`); `vault:Target` on a publish | retry once now (storage also writes a non-halt alert) |
| `vault:Replay` on an execute | rebuild |
| `vault:Slippage`, `vault:Target`, `vault:Limit`, `vault:Pricing`, `aggregator:InsufficientOutput` | skip at least one cycle |
| anything else | **halt** |

A halt stops every later cycle of that vault until an operator clears it with `python -m etesia_quant.keeper clear --alert N`, which needs neither the key nor the RPC. The monitor reports every uncleared halt as A13.

**Reconciliation drill** (`drill`). `python -m etesia_quant.keeper drill --data-dir /data --receipt deploy/testnet/d4-09-guard-setup-supply.jsonl [--yes]` runs the real `reconcile` (5.7) on an archived execute receipt: the guard setup Supply `ae59d009…1d88`, shipped in the image byte for byte as line 3 of the [guard receipts][d409]. It checks that the receipt is a successful transaction under the pinned manifest, that the archived pre-holdings reconcile, then reconciles again with 1 USDC (10,000,000 units) added to the pre-read USDC spot. With `--yes` it writes one halt row naming the flagship, whose reason starts "D6-04 drill, injected offline against a real receipt"; without `--yes` it prints the row and writes nothing. It refuses while the flagship has an uncleared halt.

### 5.6 Transaction-budget split

When `execute` of a plan with two or more actions fails simulation with `Error(Budget, ExceededLimit)` (a swap with a Blend action, or two swaps; [D6-02](d6-02-limits-v1.md) Finding 1), `_split` sends one part:

1. each swap alone, in the order of the distance its projection reaches, lowest first; a swap that the plan's sell or Withdraw funds is skipped ("Swap alone: not funded without the plan's other actions"), and a swap must pass `planner.checks` on its own projection;
2. if no swap passes, or the vault's simulation refuses every passing swap, a Blend-only plan built for the current holdings without swaps (`planner.build(..., trade=False)`);
3. nothing, if no part passes the checks and simulates.

Every part goes through the same simulation gate, and at most one part is sent per cycle. The original actions are kept under `planned` in the attempt's journal row (`attempts.context`) and in its evidence file (5.8).

### 5.7 Reconciliation (D4-06)

After every included `execute`, the keeper reads `holdings()` and the Blend position again, then fetches with `getEvents` the vault's own events and the manifest assets' transfers to or from the vault over the ledgers since the plan was built. `reconcile` requires:

- each Swap to match a transfer of exactly `amount` from the vault to the aggregator, with output at least `minimum`;
- each Supply or Withdraw to match a transfer to or from the pool;
- no vault transfer unexplained by an action;
- transfers of other transactions in the range (user deposits and exits, third-party `unwind` or `recover_supply`) to be explained by their own transfer and position events;
- a residual of at most 1 base unit per asset and holding component;
- for Blend supply, the change in bTokens to equal what the position events explain, and the supply residual to stay within the accrual bound `bTokens × |Δb_rate| / 10¹² + 1`.

Any problem records a **halt**. So do a failed post-read and a range wider than 1,000 ledgers (`MAX_RECONCILE_LEDGERS`), a margin below the roughly 4,000-ledger range at which the SDF testnet RPC returns -32001 on the XLM SAC transfer filter. Action kinds other than Swap, Supply and Withdraw halt as "not reconciled by the D4 keeper". The inputs (pre and post holdings, pool reads, other events) are archived so the check can be re-run from the evidence alone.

### 5.8 Per-attempt evidence

Every resolved attempt is written once to `/data/keeper/evidence/` as canonical JSON named by its SHA-256 (`evidence.publish`). The record holds the outcome, classification, action, ledger, the envelope, result and meta XDR, and for executes the reconciliation and its inputs. Each execute record also holds, from `split_evidence(context)`:

- `planned_actions`: the plan as built;
- `sent_actions`: the actions in this attempt's envelope, whose hash is `action_hash`;
- `split`: `null` when the plan was sent whole; after a split, `cause` ("execute simulation failed with Error(Budget, ExceededLimit): more than one action"), `part` (`swaps-alone`, the one swap sent, or `blend-only`), `rebuilt` (true when the sent Blend actions differ from the planned ones), `not_sent` and `reason`.

These fields are built from the journal context alone, so an attempt resolved by a later process records the same. Keeper attempt records use `schema_version` 2; `evidence.load` accepts version 2 only for `record` `d4-keeper-attempt`.

These files stay on the Railway volume. The public onchain record is the vault repository's receipt archive, recorded independently from RPC with `scripts/record-evidence.py` ([archive index][index]).

## 6. Target and Plan wire format

- `vault_contract.target_xdr` encodes a target as the canonical ScVal tuple `(target_v1, Target)`, byte-identical to Soroban `ToXdr`; `target_hash` is its SHA-256, which every Plan carries. The encoder rejects unknown fields, non-canonical asset order, weights not summing to 10,000, and negative numbers.
- `planner.plan_scval` encodes the vault's `Plan` type. Swap routes are built from `Hop::Soroswap` hops only.
- Both formats are pinned by golden vectors shared with the public vault repository. [`fixtures/d2-target-v1.json`][d2fix] and [`fixtures/d4-plan-v1.json`][d4fix] are byte-identical to the quant test fixtures (SHA-256 `50ee60ef…087b` and `069b7ef9…7693`). They are checked on the Rust side by [`python_rust_target_hash_golden_vector`][rs318] and [`python_rust_plan_golden_vector`][rs359], and on the Python side by `tests/test_vault_contract.py` and `tests/test_planner.py`.

## 7. Event indexer (D5-04)

`indexer.py` is keyless and read-only. Each cycle:

- **Ingests** (`ingest`, `window`) `getEvents` from its cursor to the RPC's latest ledger, in windows of 1,000 ledgers, a margin below the roughly 4,000-ledger range at which the SDF testnet RPC returns -32001 (processing limit) on the XLM SAC transfer filter. The filters (`filters`) select every event of the manifest vaults, including share-token `mint`, `transfer`, `burn` and `approve`, plus the manifest assets' SAC transfers to or from a vault. Protocol 23 transfer data, a plain amount or an `{amount, to_muxed_id}` map, is decoded.
- **Is idempotent.** Rows are keyed by the RPC event id and inserted with `INSERT OR IGNORE`. A window's rows and the cursor are committed in one transaction, so a re-run inserts nothing twice. The cursor never passes the `latestLedger` that `getEvents` itself reported.
- **Records gaps.** A cursor below the RPC's oldest retained ledger records a gap row and resumes at the oldest ledger.
- **Snapshots** (`snapshot`) each vault from simulated views: `state`, `total_assets`, `convert_to_assets(1 share)`, `holdings`, `target` and each oracle `mark`. A failed view is stored as its classified error (for example `vault:Pricing` or `oracle:Stale`). A failed ingestion still takes the snapshot, then exits non-zero.

Store: one SQLite file per network epoch, `/data/indexer/<network_epoch>.sqlite3` (WAL, `user_version` 1; other versions are refused), with tables `events` (decoded topics and data plus the raw topic and value XDR), `cursor`, `gaps` and `snapshots`. The first run starts at the earliest vault deploy ledger in the manifest. Snapshots exist from the indexer's deployment on, because views cannot be simulated at past ledgers; equity before that comes from `plan` and `unwind` events.

## 8. HTTP API and metrics routes (D5-05)

Every route except `/healthz` needs `Authorization: Bearer <ETESIA_API_KEY>`, compared in constant time (`secrets.compare_digest`). Validation errors never echo input. The Builder reaches the service only through its server-side proxy, which keeps the key on the server and allowlists the vault metrics, the Builder analytics and targets, quotes and the report files.

**Vault metrics** (`api.py`, `metrics.py`). They read the indexer database read-only; `{name}` is `flagship` or `guard`. Every response carries `vault`, `contract`, `network_epoch`, `network: "testnet"`, `status` (`fresh`, `stale` beyond 900 s, or `missing`), `as_of` and `age_seconds`. Amounts are decimal USDC (7 decimals) and shares (12 decimals); event integers beyond 2⁵³ − 1 are returned as strings. Without the manifest variables the routes return 503.

| Route | Main response fields |
| --- | --- |
| `GET /v1/vaults/{name}/summary` | `tvl_usdc`, `share_price_usdc`, `total_supply_shares`, `ledger`, `paused`, `nonce`, `target` (`epoch`, `expiry`, `assets`, `weights_bps`, `yield_bps`), `errors` (view errors), `indexer` (`next_ledger`, `latest_ledger`, `lag_ledgers`, `gaps`, freshness) |
| `GET /v1/vaults/{name}/history?limit=` | `points` (`captured_at`, `ledger`, TVL, share price, supply, `errors`); `plan_equity` (each `plan` or `unwind`: `ledger`, `tx_hash`, `nonce`, `before_usdc`, `after_usdc`, back to the deploy ledger) |
| `GET /v1/vaults/{name}/allocations?limit=` | `holdings` per snapshot (per asset: `amount`, `spot`, Blend `supply`, `collateral`, `debt`, `value_usdc` at that snapshot's oracle marks, `fraction`); `targets` (every published target with `ledger`, `tx_hash`, `hash`, weights) |
| `GET /v1/vaults/{name}/performance` | `actual`: share-price return from the `init` event's seed price (or the first snapshot) to the latest snapshot, `annualized: false`, `window_days`. Separately, `backtest`: the latest published backtest's `run_id`, window and metrics (`basis: "backtest"`), and `null` for the guard |
| `GET /v1/vaults/{name}/events?kind=&limit=` | the newest events, oldest first (`id`, `ledger`, `closed_at`, `tx_hash`, `contract`, `kind`, `topics`, `data`). Default kinds are `target`, `plan`, `unwind`, `position`, `fees` and `rewards`; `kind` may be repeated |
| `GET /v1/vaults/{name}/positions/{address}` | `shares` (from share-token mint, burn and transfer events), `shares_atomic`, `share_price_usdc`, `share_price_as_of`, `value_usdc`, `flows` (the address's deposits and withdrawals) |

**Other routes**, used by the Builder's analytics and the backtest reports:

| Route | Purpose |
| --- | --- |
| `GET /healthz` | process liveness (unauthenticated) |
| `GET /v1/status` | market snapshot coverage, refresh state, latest backtest run |
| `POST /v1/builder/targets`, `POST /v1/vault/targets`, `GET /v1/vault/weights` | allocation previews from the D1 decision for a Builder basket or the vault |
| `POST /v1/builder/simulation`, `POST /v1/basket/analytics`, `POST /v1/correlations` | portfolio simulation and basket statistics |
| `GET /v1/tokens`, `/v1/tokens/{token}/last-close`, `/v1/tokens/{token}/sharpe`, `/v1/market-caps`, `/v1/prices/live` | catalog, prices and market data |
| `POST /v1/quotes` | a mainnet Soroswap aggregator quote, validated (read-only; nothing is built or signed) |
| `POST /v1/testnet/liquidity` | testnet pool realign and top-up before a Builder build's swaps; its own bearer key, 404 unless enabled; **signs and sends** (section 10) |
| `GET /v1/vault/backtests/latest`, `/{run_id}/track`, `/{run_id}/performance`, `/{run_id}/files/{pdf\|xlsx}` | immutable backtest runs and their reports |
| `GET /docs`, `GET /openapi.json` | API documentation (authenticated) |

## 9. Monitor and alerts (D6-03)

`monitor.py` checks every manifest vault each run (`Monitor.states`). It uses read-only simulated views, `getLedgerEntries`, `getLedgers`, and the keeper journal, indexer database and liquidity ledger, each opened read-only.

A check that cannot read its inputs reports its key open with `check failed: …`, so a failure is never taken as healthy. An extra key, `rpc`, opens when `getLatestLedger` fails. Each alert key opens once and resolves once in `monitor_alerts`, which records the Telegram message id of each delivery; an undelivered message is retried every run (`Alerts.apply`). Messages never contain the RPC URL's path or query (`redact`), and the bot token is never logged.

| Alert | Opens when | Default threshold |
| --- | --- | --- |
| A1 | The keeper vault's latest heartbeat started too long ago, a cycle is still `running`, or there is no heartbeat or journal. Flagship only: the guard has no keeper loop | 2,700 s (3 × 900 s); `running` beyond 600 s |
| A2 | `target().epoch` is older than the threshold by ledger time, or no target is readable | 93,600 s |
| A3 | The ledger of `state().last_execution` closed longer ago than the threshold, or no plan has executed | 93,600 s |
| A4 | `state().paused` is set | — |
| A5 | A `state`, `config`, `total_assets` or `holdings` simulation fails; the vault error is named | — |
| A6 | A provider `mark` is older than the threshold, or fails | the vault's `max_price_age` + 300 s (one Reflector round of grace) |
| A7 | A manifest contract's wasm hash differs (`A7:code`), or the admin of Reflector, the Blend pool or the aggregator copy differs from its pin | — |
| A8 | The aggregator copy's `config()` differs from the manifest | — |
| A9 | A vault executor's XLM balance is below the threshold | 100 XLM |
| A10 | A manifest contract's instance or code entry has fewer ledgers to live than the threshold, or is missing | 51,840 ledgers |
| A11 | The indexer database or its cursor is missing (`A11:not-running`); the last ingested ledger lags the chain (`A11:lag`); or a gap was recorded in the last 24 h (`A11:gaps`) | 360 ledgers |
| A12 | The Blend pool's USDC supply emission entry is not null | null expected |
| A13 | A keeper journal alert row: a halt stays open until `keeper clear`; a non-halt row is reported for 24 h | — |
| A14 | healthchecks.io dead-man. The monitor pings only while A1 is clear, so a stopped process, a failing monitor or a stale keeper all stop the pings; the provider raises the alert | outside the process |
| A15 | Testnet liquidity, evaluated only when `ETESIA_LIQUIDITY_ACCOUNT` is set; never a halt. `A15:float`: the account's USDC balance is below the threshold. `A15:depth:<pair>`: a 100 USDC trade either way at the mid moves a manifest pair above the threshold. `A15:claims`: the UTC day's faucet claims in the quant liquidity ledger reached the daily cap. `A15:failures`: a liquidity attempt of any kind (add, realign, claim, forward, remove) included-failed or was never included, or a faucet key failed, in the last 24 h | 1,000 USDC (`min_float_usdc`); 100 bps (`max_depth_impact_bps`) |

`ETESIA_MONITOR_OVERRIDES` (`overrides`) can lower a threshold or replace a pin for one deploy so that an alert can be induced once. Every alert caused this way carries `[threshold-induced: …]` or `[pin mismatch induced; no onchain change]` in its text, and unknown keys stop the run. `python -m etesia_quant.monitor check` prints the current states without writing or sending anything.

## 10. Testnet liquidity

`liquidity.py` runs on testnet only. The testnet Soroswap pairs are shallow, so a vault or Builder swap could move them by several percent. Before Etesia's own swaps, the service moves a pair that is off the Reflector mid back to it (realign) and adds Etesia's own tokens to the pair (top-up). Liquidity added this way is Etesia-supplied, not market depth, and a realign is Etesia's own swap, not market activity.

**Depth arithmetic** (`impact_bps`, `needed_scale`, `at_scale`). A pair's tokens and reserves are read with simulated `token_0`, `token_1` and `get_reserves`. A swap's impact is `γa / (R_in + γa)` with γ = 997/1000, which equals the Builder's `1 − (out/in)/(refOut/refIn)`. A top-up scales both reserves by the same factor `f`, at the current ratio, so the pair's mid price does not move; `f` is the smallest factor that brings every requested leg on the pair within its impact limit:

- **Builder request:** 1% (`MAX_IMPACT_BPS` 100).
- **Keeper:** the impact its slippage budget still admits, `1 − (1 − b) / (γ (1 − φ) (1 + d))` less 5 bps and at most 1%, with `b` the vault's `max_slippage_bps` less 10 bps, `φ` the v3 copy's `protocol_bps` and `d` the pair's adverse deviation from Reflector (`keeper_impact_bps`), about 45 bps at parity. When the deviation alone uses the budget, the pair needs a realign.

**Realign** (`Provider.top_up`, `Provider.realign`). Only right before Etesia's own swaps, and only for a pair one of them uses: when the pair's mid is off the Reflector mid against the swap, the float first sells the other token into that pair once, so that the mid returns to the Reflector mid. A favourable deviation is left alone.

- **Trigger** (`realign_needed`, `deviation`). The keeper realigns when the deviation alone uses its budget (`keeper_impact_bps` gives None); the endpoint when the deviation against a swap is strictly above 100 bps. `d` is the out side's value over the in side's at Reflector, less 1. The keeper passes the marks of its own cycle; the endpoint reads the first manifest vault's oracle (`Provider.marks`).
- **Amount** (`realign_amount`). With `a` the reserve of the token sold into the pair, `Δ` is the positive root of `γΔ² + (1 + γ)aΔ + a²d/(1 + d) = 0`, floored, so that after the swap the mid equals the Reflector mid.
- **Transaction.** Router `swap_exact_tokens_for_tokens` on that pair only, from and to `etesia-liquidity`, minimum output 10 bps below the expected output. It goes through the same simulate, source-only auth, journal-before-send and settle path as an add, as an `attempts` row of kind `realign` valued at Reflector, and is archived as an `a102-liquidity-realign` evidence record with the marks, the reserves before and after, the expected and minimum output and the envelope.
- **Caps** (`REALIGN_DAILY_USDC`, `Ledger.realigned`). 3,000 USDC a UTC day for the keeper and 1,000 USDC for the endpoint, each counted on its own (successful realigns, the sold amount at Reflector) and apart from the top-up caps. The sale is also bounded by the float (for the keeper, less the Builder's reserve); short of either, it is scaled down with the note "realign scaled to N% by the float" or "… by the daily realign cap".
- **Never fails the caller.** An exception in the realign becomes a note and the top-up runs on the pool as it is. A realign sent but not confirmed stops further sends in that call ("realign unconfirmed; nothing more is sent"); the next call settles it first. A needed realign not sent in full makes the status at least `partial`.

**Transactions** (`Provider.add_liquidity`, `submit`, `settle`). Each `add_liquidity` goes to the manifest's Soroswap router from `etesia-liquidity`, with minimums 10 bps below the desired amounts and a `maxTime`. The envelope is simulated first; nothing is signed if simulation fails or if any authorization entry needs a signature other than the source account's. As in the keeper, the hash is journalled before `sendTransaction`, one attempt may be unresolved at a time (`Busy`), a lost response is settled by polling `getTransaction`, and a resend reuses the same envelope. Each add is archived once as an `a98-liquidity-topup` evidence record with the reserves and each leg's impact before and after, the desired, minimum and added amounts and the LP units minted.

**One top-up** (`Provider.top_up`). For each manifest pair the legs trade on, the addition is scaled down at the same ratio when the float (read onchain, less 100 XLM kept for fees and reserves), the caller's reserve, the 2,000 USDC per-pair cap or the caller's USDC budget does not cover it. The result is `ready` (no pair needed liquidity), `topped_up`, `partial` or `unavailable`, with notes. The keeper, the endpoint and the refill share one non-blocking writer lock.

**Keeper hook** (`Keeper._top_up`; on only with `ETESIA_LIQUIDITY_KEEPER=1`). After the target is published and the venue read, and before any quote, the keeper sizes the legs it intends without quotes (`planner.intended`), realigns and tops up their pairs to its own limit. It keeps the Builder's reserve (1,000 USDC, 1,000 tRESERVE, 7,500 XLM) and needs at least 30 s in the Reflector window after keeping 30 s for the execute and 20 s for quotes. It never halts the vault, never writes keeper `attempts` and never changes the cycle status: a failure or a held lock is a skip reason, each realign is a skip line `liquidity: realigned <pair>: sold … for …, deviation <before> -> <after> bps (tx …)`, and a low float or two failed adds in a row is a non-halt journal alert, reported by the monitor as A13 for 24 h.

**Endpoint** `POST /v1/testnet/liquidity` (`serve`; request model in `api_models.py`). It answers 404 unless `ETESIA_LIQUIDITY_ENABLED=1` and `ETESIA_LIQUIDITY_API_KEY` is set, and accepts only that key, not `ETESIA_API_KEY`. The Builder calls it through its server-side proxy before quoting a testnet build. A request names the owner account, a build id and 1 to 4 swaps in base units:

- swaps only between manifest assets on manifest pairs; each at most 101 USDC and all at most 202 USDC at the pool mid;
- the owner must hold each sell's input, plus the USDC for buys beyond what the sells raise (SAC balances read onchain);
- a repeat within 15 minutes returns the stored response;
- limits: per owner one request every 5 s, one top-up every 10 minutes and 5 a UTC day; per client 10 top-ups a day; in all 60 requests a minute, 20 top-up transactions and `ETESIA_LIQUIDITY_DAILY_USDC` (default 5,000 USDC) a day. The per-owner and per-client limits count requests that sent an add or a realign transaction; with the daily top-up cap reached, a needed realign still runs under its own cap.

A pair whose deviation against a swap is above 1% is realigned, and a pair where a swap's impact is above 1% is topped up to 1%. The response gives the status, each swap's impact before and after, each realign and addition with its hash, a reason and the float. Marks older than the vault's `max_price_age` skip the realign ("the Reflector marks are older than max_price_age"). Errors are 401, 409 (lock held or attempt in flight), 422 (outside the manifest, over the caps or beyond the owner's holdings), 429 with `Retry-After`, and 503 (manifest, secret or RPC unavailable).

**Faucet refill** (`refill`, `claim`, `forward_and_merge`, `reconcile`). Hourly, and only while the float holds less than 3,000 USDC, the refill claims from the Blend testnet faucet until it holds 10,000 USDC, at most 10 claims a UTC day (`ETESIA_LIQUIDITY_CLAIMS_PER_DAY` can lower this, not raise it; failed claims count) and 900 s apart. Each claim uses a fresh key derived from the liquidity secret by HMAC-SHA256 over the epoch and an index, so no key is stored (`fresh_key`). Friendbot funds it. The faucet's answer is checked before any signature (`validate_claim`): a plain transaction whose source and fee payer is the manifest USDC issuer, with only that issuer's USDC, BLND, wETH and wBTC `change_trust` by the fresh key and payments to it. A `tx_bad_seq` is fetched again once. One transaction from the fresh key then pays its USDC to `etesia-liquidity`, returns BLND, wETH and wBTC to the issuer, removes the trustlines and merges the account into `etesia-liquidity`. Each refill first settles attempts left in flight and forwards claims that landed late. Without `ETESIA_BLEND_FAUCET_URL` the refill does nothing. Each claim uses the faucet issuer's next sequence number, so it can invalidate another user's claim pending at the same moment.

**Operator commands** (`main`): `python -m etesia_quant.liquidity status` (float, pair reserves, the day's claims and spend, unresolved attempts), `refill`, and `remove --pair C… [--liquidity N] [--yes]`, which prints the router `remove_liquidity` plan for the account's LP units (all by default, pro-rata minimums less 10 bps) and sends it only with `--yes`.

**Store** (`Ledger`). `/data/liquidity/<network_epoch>.sqlite3` with tables `attempts` (every add, realign, claim, forward and removal: hash, envelope, amounts, USDC value, requester, `maxTime`, outcome, ledger), `faucet` (key index, public key, state `derived`, `funded`, `claimed`, `merged` or `failed`, claim hash), `requests` (stored responses with a `sent` flag; the client header is hashed) and `arrivals`. A ledger created before the realign kind existed is migrated in place, rows and their order kept.

**Setup outside the service.** The `etesia-liquidity` account, its trustlines, the tRESERVE float, the first faucet claims and the pair seeds were made with `etesia-vault/scripts/testnet-deploy.py liquidity` (vault `14c2a48`). Those claims are journalled by that script, so the refill's daily cap and `A15:claims`, which count `faucet` rows of the quant ledger, do not include them.

## 11. Configuration names

Names only; values live in Railway service variables. Secrets are marked.

| Name | Read by | Role |
| --- | --- | --- |
| `ETESIA_API_KEY` (secret) | API | bearer key for every route but `/healthz` |
| `ETESIA_DATA_DIR` | API, loops | persistent data root |
| `ETESIA_REFRESH_ENABLED`, `ETESIA_KEEPER_ENABLED`, `ETESIA_INDEXER_ENABLED`, `ETESIA_MONITOR_ENABLED` | API | start each background loop; `ETESIA_KEEPER_ENABLED=0` is the keeper's kill switch |
| `ETESIA_MANIFEST`, `ETESIA_MANIFEST_SHA256`, `TESTNET_RPC_URL` | keeper, indexer, monitor, metrics routes, liquidity | testnet manifest path, its pin, and the testnet RPC |
| `ETESIA_EXECUTOR_SECRET` (secret) | keeper only | flagship executor key; checked against the manifest executor |
| `ETESIA_KEEPER_VAULT` | keeper, monitor | manifest vault the keeper runs and A1 watches |
| `ETESIA_KEEPER_SCENARIO` | keeper | scenario target (`flat`, `xlm-1500`); unset in normal operation |
| `ETESIA_KEEPER_FAULT` | keeper | restart-test fault hook; testnet only, unset in normal operation |
| `SOROSWAP_API_KEY` (secret) | API, keeper, pool collector | Soroswap API quotes |
| `COINGECKO_API_KEY` (secret) | API, refresh, pool collector | market data |
| `ETESIA_MONITOR_OVERRIDES` | monitor | induce an alert once; unset in normal operation |
| `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` (secret) | monitor | alert delivery |
| `HEALTHCHECKS_PING_URL` (secret) | monitor | A14 dead-man ping |
| `STELLAR_RPC_URL` | pool collector | mainnet RPC for pool data (research, read-only) |
| `RAILWAY_GIT_COMMIT_SHA` | backtest | revision recorded in backtest provenance |
| `ETESIA_LIQUIDITY_ENABLED` | API | serve `POST /v1/testnet/liquidity` and start the hourly refill loop |
| `ETESIA_LIQUIDITY_SECRET` (secret) | keeper hook, endpoint, refill | `etesia-liquidity` key; also the seed of the fresh faucet keys |
| `ETESIA_LIQUIDITY_API_KEY` (secret) | API | bearer key of `POST /v1/testnet/liquidity` only; the same value is the Builder's server-side variable |
| `ETESIA_LIQUIDITY_KEEPER` | keeper | `1` turns on the keeper hook (realigns and top-ups) |
| `ETESIA_LIQUIDITY_ACCOUNT` | monitor | public key of `etesia-liquidity`; unset means no A15 |
| `ETESIA_BLEND_FAUCET_URL`, `FRIENDBOT_URL` | refill | Blend testnet faucet endpoint (unset: the refill does nothing) and an optional Friendbot override |
| `ETESIA_LIQUIDITY_CLAIMS_PER_DAY`, `ETESIA_LIQUIDITY_DAILY_USDC` | refill and monitor; API | lower the 10 claims a day (never raise it); the endpoint's daily USDC cap (default 5,000) |

## 12. Test suite

At `1e8d2aa`, `.venv/bin/python -B -m unittest discover -s tests` in `etesia-quant/` (Python 3.14.7) gives **`Ran 498 tests`, `OK`** (2026-11-14). Tests use injected fakes for the RPC, quotes, clock and Telegram; none touches the network. The output also contains log lines such as `keeper halt on …` from simulated halt scenarios and Starlette deprecation warnings; none is a failure.

| Module | Tests | Covers |
| --- | --- | --- |
| `test_keeper.py` | 82 | publish then execute with reconciliation; lost publish or execute response then restart executes once; resend of the same envelope; expiry and never-included attempts; every `sendTransaction` error path; the twice-rule halt; resolve-once; halt clearing and the reconciliation drill; skip and Pricing waits; Reflector window polling and deadline caps; 429 retry within the deadline; storage re-simulation; failed post-read and too-wide range halt; publish Target retry and Replay halt; third-party unwind; the budget split (one swap alone from a plan over the budget, a funded swap not tried alone, the next swap after the vault refuses one, the Blend-only fallback) and its evidence fields; the liquidity hook after the publish and before any quote, its skips, realign lines and non-halt alerts; fault hook testnet-only; error classification; reconciliation with deposits, accrual, third-party transfers and the 1-unit tolerance |
| `test_planner.py` | 21 | distance mirror; plan order; leg sizing on live quotes; cooldown, leg cap, turnover and loss windows, spot floors and headroom, accrual margin, yield cap, 40% cap, falling distance and action count; the Supply reserve and lone leg-cap swaps within it; the lone Withdraw; intended legs equal the built swaps; Python side of the Plan golden vector |
| `test_routing.py` | 9 | quote request shape; HTTP and transport errors; route translation and rejection of off-manifest quotes; minimum-output rounding; admissibility boundary; submission window; constant-product solve |
| `test_target_builder.py` | 11 | pinned testnet manifest only; asset consistency; network check; `t2-map-v1` vectors; settlement identity; Rust target vector and yield limit; valid and blocked decisions; validator mirror |
| `test_vault_contract.py` | 4 | reserve split; shared Rust target XDR vector; rejection of ambiguous targets |
| `test_triggers.py` | 8 | Drift and Signal bands; Scheduled once; marks inside and outside the band; blocked decision keeps stored weights; failed read skips; scenario targets are record-only |
| `test_d4_09_guard_cases.py` | 26 | guard harness: expected codes, case order, preconditions, twin footprints, source auth entry, nothing sent without a matching simulation, journalling before send, Pricing waits, signature check against the manifest address |
| `test_indexer.py` | 15 | golden decoder over the vault's local and testnet fixtures; protocol 23 muxed transfers; backfill and re-run with no duplicates; retention gap; restart from the cursor; lagging `getEvents` node; schema refusal; snapshot errors; loop flag; metrics: freshness, missing data, history, allocations, performance, events by kind, big integers, positions |
| `test_monitor.py` | 29 | every check firing on its condition and silent when healthy; failed reads reported open; A6 grace; A1 heartbeat cases; A13 halt and notice expiry; A11 against the indexer's own schema; A10 missing entries; A15 keys and their overrides; RPC URL redaction; induced labels and override validation; open-once and resolve-once with message ids; delivery retries without logging the token; ping only while A1 is clear; RPC outage; loop flag |
| `test_liquidity.py` | 44 | impact against the integer `amount_out`; additions at the ratio keep the mid; the router floor; the keeper's limit from its budget, fees and deviation; envelope minimums and source-only auth; journal before send; restart while in flight; the shared lock; the Builder reserve kept from the keeper; scaling under the USDC cap; the float read onchain; realign math landing the mid on Reflector, its keeper and endpoint triggers, a favourable deviation left alone, the daily caps per requester kind, a short float, a failed realign as a note, a landed realign kept when its record fails; ledger migrations; faucet envelope validation and refused foreign operations; claim, forward and merge; `tx_bad_seq` refetch; late claims; the persisted daily cap; a malformed secret kept out of stderr |
| `test_api.py`, `test_builder_api.py`, `test_token_api.py`, `test_analytics_window.py` | 46, 15, 25, 6 | authentication; Builder baskets and caps; quote validation; reports; refresh scheduling, failure and recovery; snapshots and restart; catalog, prices and analytics; the liquidity endpoint (off without its switch and key, its own key, realign before top-up, top-up to 1%, ready without a transaction, manifest pairs, caps and holdings, idempotent repeat, owner, client and global limits, daily caps, stale or missing marks, 409 and 503) |
| `test_d1.py`, `test_history.py`, `test_data.py`, `test_pools.py`, `test_evidence.py`, `test_shadow.py`, `test_universe.py` | 63, 35, 14, 36, 4, 2, 3 | D1 numerics, signals, backtest and reports; history ingestion; storage; pool data; immutable evidence; universe |

## 13. Scope of the code

Items that matter at mainnet are listed in the [D6-05 mainnet readiness checklist](d6-05-mainnet-readiness-checklist.md).

- **Testnet only.** The keeper, indexer, monitor and liquidity module load only a testnet manifest. There is no mainnet keeper.
- **Soroswap pools, one hop.** Quotes request `protocols: ["soroswap"]` with one hop, and routes must use a pair in the vault's `route_contracts`. On mainnet most BTC, ETH and AQUA liquidity sits in other venues ([D6-02](d6-02-limits-v1.md) Finding 2).
- **Testnet assets.** The manifest has USDC, tRESERVE and XLM; `t2-map-v1` maps the decision's XLM exposure (section 3).
- **Per-transaction budget.** Because of the vault's per-execute valuation cost, a swap with a Blend action, or two swaps, exceed one execute's transaction budget ([D6-02](d6-02-limits-v1.md) Finding 1); the keeper sends one swap per execute (5.6), and Supply keeps a 1% reserve below the yield limit for those lone swaps (5.3).
- **Reconciliation scope.** Swap, Supply and Withdraw are reconciled. Any other action kind halts, and so does a range of more than 1,000 ledgers between the plan and its post-read.
- **One keeper vault.** The keeper loop and A1 cover the flagship. The guard is checked by A2–A13 and driven by the D4-09 harness.
- **Indexed history.** Snapshots start at the indexer's deployment; earlier equity comes from `plan` and `unwind` events.
- **Testnet liquidity.** Etesia is an LP of the pairs it tops up, including the third-party XLM/USDC pair. Top-ups, realigns, claims and removals are outside keeper reconciliation and the indexer, which follow vault transfers; A15 covers them. The faucet claims draw on Blend's shared testnet faucet. The 2026-12-16 testnet reset wipes the float and the LP positions.

## 14. Evidence index

Receipts are in the public vault archive at `5807b85` ([index][index]).

| Component | Code | Evidence |
| --- | --- | --- |
| Manifest and network pin | `network.py` | [Archive index][index] (same manifest SHA-256) |
| Decision, target builder and triggers | `strategy.py`, `target_builder.py`, `triggers.py` | [`t2-d4-06-receipts.json`][d406] (first deployed-keeper publishes); [`t2-d4-07-scheduled-decision-2026-11-12.json`][d407] and [`t2-d4-07-scheduled-0100-2026-11-12.json`][d407b] (the 01:00 Scheduled decision) |
| Quotes, routes and plans | `routing.py`, `planner.py` | D4-02 probes in [`d4-testnet-probes.json`][probes] |
| Keeper submission, journal, reconciliation | `keeper.py` | [`t2-d4-06-receipts.json`][d406], [`t2-keeper-cycles-2026-11-10-to-11.json`][kc1], [`t2-keeper-cycles-2026-11-11-evening.json`][kc2], [`t2-keeper-cycles-2026-11-12.json`][kc3]; reconciliation records [`t2-d4-10-reconciliation/`][recon] |
| Blend yield | `planner.py`, `keeper.reconcile` | [`t2-d4-08-receipts.json`][d408]; [D4-08 NAV read](evidence/d4-08-nav-read-2026-11-14.md) |
| Guard rejections | `scripts/d4-09-guard-cases.py`, `keeper.classify` | [`d4-09-guard-pre-2026-12-16-reset.jsonl`][d409] |
| Acceptance run, budget split, kill/restart | `keeper.py` | [`t2-d4-10-step1.json`][s1], [`step2`][s2], [`step2b`][s2b], [`step2c-drill`][s2c] |
| Wire formats | `vault_contract.py`, `planner.plan_scval` | Shared vectors in the vault [`fixtures/`][fixtures] (section 6) |
| Indexer | `indexer.py` | Decoder fixtures [`fixtures/d5-04-testnet/`][d504] |
| Metrics API | `api.py`, `metrics.py` | [D5-05 latency record](evidence/d5-05-latency-2026-11-12.md) |
| Monitor and alerts | `monitor.py` | [D6-03 alert evidence](d6-03-alert-evidence.md) |
| Runbook drills | `keeper.py` (`clear`, `drill`), `indexer.py`, `monitor.py`, `liquidity.py` (`remove`) | [D6-04 runbook drills](d6-04-runbook-drills.md) |
| Testnet liquidity | `liquidity.py`, `keeper._top_up`, `api.py`, `monitor.py` (A15) | Setup and seed [`t2-liquidity-setup-2026-11-12.json`][lsetup], [`t2-liquidity-faucet-keys-2026-11-12.json`][lkeys]; removal [`t2-liquidity-remove-2026-11-12.json`][lremove]; automated refills [`t2-liquidity-refills-2026-11-13.json`][lrefills]; first keeper realign [`t2-a102-realign-2026-11-13.json`][lrealign]; further refills and the first Builder-requested realign [`t2-liquidity-refills-and-builder-realign-2026-11-13.json`][lbuilder] |

[vault]: https://github.com/Etesia-Research/stellar_vault
[index]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-index.md
[d409]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/d4-09-guard-pre-2026-12-16-reset.jsonl
[d406]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-06-receipts.json
[d407]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-07-scheduled-decision-2026-11-12.json
[d407b]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-07-scheduled-0100-2026-11-12.json
[probes]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/d4-testnet-probes.json
[kc1]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-keeper-cycles-2026-11-10-to-11.json
[kc2]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-keeper-cycles-2026-11-11-evening.json
[kc3]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-keeper-cycles-2026-11-12.json
[recon]: https://github.com/Etesia-Research/stellar_vault/tree/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-10-reconciliation
[d408]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-08-receipts.json
[s1]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-10-step1.json
[s2]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-10-step2.json
[s2b]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-10-step2b.json
[s2c]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-d4-10-step2c-drill.json
[fixtures]: https://github.com/Etesia-Research/stellar_vault/tree/5807b8535ed26224e7e6155ac65a6e4581d81752/fixtures
[d2fix]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/fixtures/d2-target-v1.json
[d4fix]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/fixtures/d4-plan-v1.json
[rs318]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/contracts/vault/src/tests.rs#L318
[rs359]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/contracts/vault/src/tests.rs#L359
[d504]: https://github.com/Etesia-Research/stellar_vault/tree/5807b8535ed26224e7e6155ac65a6e4581d81752/fixtures/d5-04-testnet
[lsetup]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-liquidity-setup-2026-11-12.json
[lkeys]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-liquidity-faucet-keys-2026-11-12.json
[lremove]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-liquidity-remove-2026-11-12.json
[lrefills]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-liquidity-refills-2026-11-13.json
[lrealign]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-a102-realign-2026-11-13.json
[lbuilder]: https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-liquidity-refills-and-builder-realign-2026-11-13.json
