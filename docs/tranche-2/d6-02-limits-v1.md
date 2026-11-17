# D6-02 limits-v1: proposed mainnet limits

limits-v1 is the final calibration of rebalance thresholds, slippage caps and concentration limits for mainnet: each parameter with its value and derivation, a protocol-30 resource profile and a concentration review. Version limits-v1, 2026-11-11, from mainnet data of 2026-11-11. Testnet vaults keep their D4-03 values. The mainnet values are set at the review of the [D6-05 checklist](d6-05-mainnet-readiness-checklist.md).

Citations are workspace-relative `path:line`. Vault lines are at `6c8a723`, quant lines at `c712ea6`. The D6-02 vault files are in vault `0bcf65b`, pushed on 2026-11-11 (`origin/master` `e45ab76`). `etesia-quant/data/` is a local data directory, outside the repository.

## Data used

| Source | What it gives | Where |
| --- | --- | --- |
| Local protocol-30 measurement, 2026-11-11 | Real local `execute` receipts with their `core_metrics`: CPU, memory, read and write entries and bytes, event bytes, envelope size, declared resources, fee and call counts. Each plan is also simulated through RPC; RPC 30.0.0 returns no simulation `cost`, so simulation gives no memory figure. Two runs of one script (sha256 `027404ee…ca19`) at vault `6c8a723` plus the D6-02 files, on pinned Core 30.0.0 and RPC 30.0.0 binaries (no container) | Script `etesia-vault/scripts/d6-local-resources.py`; protocol-30 profile `etesia-vault/docs/evidence/d6-02-local-resources.json` (15:01 UTC, 64 scenarios); diagnostic profile `etesia-vault/docs/evidence/d6-02-local-resources-raised.json` (15:07 UTC, 17 scenarios) with `etesia-vault/fixtures/d6-02-raised-limits.json`; method in `etesia-vault/docs/local-rpc.md`, D6-02 section. |
| Live testnet limits, read-only, 2026-11-11 13:28 UTC, ledger 5,676,019, protocol 30 | Per-transaction limits: 400M instructions, 41,943,040 B memory, 200 read and 200 write entries, 200,000 read B, 132,096 write B, 16,384 event B, 132,096 tx B, 400 footprint entries. They are identical to the local profile. One read-only simulation of the testnet provider's `mark(XLM)`: 5,325,221 instructions, against 5,033,987 for the local Reflector copy (`mark_simulated_instructions`) | `etesia-vault/docs/evidence/d6-02-testnet-limits.json`; local figure from `d6-02-local-resources.json` |
| Live testnet flagship, 2026-11-11 | Keeper cycles 55–62: a v3 swap plus a Blend Supply or Withdraw fails simulation with `Error(Budget, ExceededLimit)`. The swap alone takes 87.9M instructions and the Supply alone 82.0M. After the quant `7beea26` split, the plans `3b03a8ea…f3de5` and `4a5e3b21…7d66` executed | [Receipt archive](https://github.com/Etesia-Research/stellar_vault/blob/5807b8535ed26224e7e6155ac65a6e4581d81752/docs/evidence/t2-index.md), ledgers 5,677,199 and 5,677,384 |
| Mainnet collector run, 2026-11-11 12:45–13:06 UTC, ledgers 65,405,942–65,423,222 | Soroswap, Aquarius, Phoenix and SDEX AMM inventory (218 / 315 / 12 / 39,702 pools), reserves, quote-derived USD marks (XLM 0.148105 from CoinGecko) and rolling 24 h volume. Status `partial` (exit 2), the documented outcome when some pools have unknown TVL; `errors` is empty. Read-only; nothing signed | Quant `520a933`: `uv run --locked --env-file .env python -m etesia_quant.pools collect --directory data/pools/2026-11-11`. Output in `etesia-quant/data/pools/2026-11-11/` |
| September mainnet snapshot, 2026-09-17 | Comparison only | `etesia-quant/data/pools/latest.parquet` |
| Testnet Reflector XLM/USD rounds, 2026-11-03 to 2026-11-10 (1,956 rounds) | Feed age: time-weighted p50 197 s, max 3,037 s. XLM move by horizon, p50/p95/p99/max in bps: 5 min 9/37/59/201; 10 min 15/55/98/277; 15 min 18/73/125/307; 1 h 37/136/331/514 | `etesia-vault/docs/evidence/d4-testnet-probes.json`, key `reflector` |
| Strategy backtest d1-v5 (15% budget), 2020-01-01 to 2026-09-17 | Per-day turnover p50 0.11%, p99 3.8%, p99.9 9.9%, max 12.1%. Leg/equity p99 2.6%, p99.9 5.2%, max 6.7%. At most 4 legs a day. Excludes two outliers: the initial deployment and the 2025-07-26 USTRY reserve switch (63%). The strategy now runs a 30% budget (d1-v6); its figures here are the d1-v5 figures ×2 | `etesia-quant/data/backtests/shx-excluded-v5/vault_2020-01-01_2026-09-17.json.gz` |

## Finding 1: the protocol-30 memory ceiling

**Summary.** With the current contract, a vault holding 2 non-USDC assets fits **one swap per execute** that leaves it holding at most 2 (W3: buying a third fails) under the standard simulate-then-send flow, and the 8-action ceiling (`etesia-vault/contracts/vault/src/execution.rs:395`) cannot be reached. XLM counts, because valuation prices every nonzero holding (`etesia-vault/contracts/vault/src/pricing.rs:168-176`). The testnet flagship holds XLM and tRESERVE, so it is in this state. A vault holding 3 or more non-USDC assets fails simulation for even one one-hop swap (local protocol-30 node, T3–T5). **A six-asset vault therefore cannot rebalance through the keeper's flow on the current contract.** This is a mainnet item in the [D6-05 checklist](d6-05-mainnet-readiness-checklist.md#findings-from-d6-02), not a parameter to tune.

**Live testnet, 2026-11-11.** Every flagship cycle under `ETESIA_KEEPER_SCENARIO=flat` (cycles 55–62) skipped: a plan mixing one v3 swap with a Blend Supply or Withdraw failed simulation with `Error(Budget, ExceededLimit)`. Each action alone fits: the swap alone takes 87.9M instructions and the Supply alone 82.0M, far below the 400M limit, so CPU is not the binding limit. Quant `7beea26` split the plan: the swaps alone, else the Withdraw alone, each gated by `planner.checks()` and simulation (`etesia-quant/src/etesia_quant/keeper.py:55,580`). The split plans `3b03a8ea…f3de5` (cycle 63) and `4a5e3b21…7d66` (cycle 64) executed. On 2026-11-13, read-only simulations on the flagship gave the same result for two 0.4 USDC one-hop swaps in one execute (`Error(Budget, ExceededLimit)`, events ending after the first swap) and for the planner's `[Withdraw, Swap]`; one 1.0 USDC swap alone passes at 87,791,477 instructions. Since quant `1e8d2aa` the keeper sends one swap per execute ([quant code overview 5.6](tranche-2-quant-code-overview.md#56-transaction-budget-split)).

**Local setup.** Six SAC assets and all 15 universe pairs on the mainnet Soroswap code (100M a side). Prices come from the Etesia pricing provider over a local copy of the **testnet Reflector code** (`25fc3060…`, v7, 20,417 B; mainnet runs Reflector v6, 19,169 B, D3 manifest), with a USD round and a USDC denominator leg, the testnet provider's shape; the copy holds one price round for six assets. The testnet Blend v2 pool, backstop and Comet code are deployed; the pool runs in status 2 (its backstop has no deposits), and Supply and Withdraw follow the status-0 path. The vault WASM is `ac188cbf…`, identical to the testnet vaults, with the flagship's limits except a 1-ledger cooldown. Two profiles are used. The protocol-30 profile equals live testnet. The diagnostic profile raises only instructions (2B), memory (400 MiB) and event size (128 KiB), so every plan applies and its receipt shows what it **needs**. Headroom is always stated against protocol 30.

**Scope of "worst case".** The measured shapes use at most 2 hops and 2 routes per swap. The keeper sends 1 hop and 1 route (`etesia-quant/src/etesia_quant/routing.py:45,145`), and the mainnet API quote allows 2 hops (`etesia-quant/src/etesia_quant/api.py:135`). The contract allows up to 4 hops per route and 15 routes per swap (`etesia-vault/contracts/vault/src/execution.rs:180,190`). At par marks, 3 Soroswap hops leave under 1 bps for price impact under a 100-bps floor (fees compound to 99.6 bps), and 4 hops cannot meet it (129 bps).

**What an 8-action execute needs when applied** (diagnostic profile; the vault holds all five non-USDC assets and a Blend supply):

| Plan | CPU (of 400M) | Memory (of 41.9 MB = 40 MiB) | Event bytes (of 16,384) | Read / write entries (of 200) | Write bytes (of 132,096) | Envelope B | Fee (stroops) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 8 one-hop swaps | 219.4M (55%) | 57.8 MB (**138%**) | 18,828 (**115%**) | 52 / 29 | 11,548 | 13,444 | 186,361 |
| 8 two-hop swaps via XLM | 239.4M (60%) | 62.0 MB (**148%**) | 28,516 (**174%**) | 52 / 28 | 11,340 | 16,772 | 222,514 |
| 7 two-hop + Blend Supply | 232.2M (58%) | 60.2 MB (**143%**) | 25,900 (**158%**) | 54 / 31 | 12,216 | 15,736 | 215,598 |
| Blend Withdraw + 6 two-hop + Supply (planner shape at `maxHops` 2) | 224.4M (56%) | 58.2 MB (**139%**) | 23,276 (**142%**) | 54 / 31 | 12,216 | 14,424 | 203,923 |
| Blend Withdraw + 6 one-hop + Supply (planner shape at `maxHops` 1) | 209.1M (52%) | 55.1 MB (**131%**) | 15,956 (97%) | 54 / 32 | 12,424 | 11,928 | 176,873 |
| 7 one-hop + Blend Supply | 214.4M (54%) | 56.5 MB (**135%**) | 17,396 (**106%**) | 54 / 32 | 12,424 | 12,824 | 184,032 |
| 8 swaps, each split over a direct and a via-XLM route | 261.0M (65%) | 66.9 MB (**160%**) | 38,868 (**237%**) | 64 / 40 | 15,156 | 22,404 | 283,801 |

Memory binds first, then event size. CPU stays at 52–65%. Read and write bytes stay at or below about 11%, entries at or below about 32%, envelopes at or below about 17%.

**Cost grows with holdings, not only with legs** (one one-hop buy, diagnostic profile):

| Non-USDC assets held, then a buy | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| Memory when applied (of 41.9 MB) | 11.9 MB (28%) | 19.9 MB (47%) | 26.5 MB (63%) | 33.1 MB (79%) | 39.7 MB (95%) |
| CPU | 42.7M | 71.6M | 95.6M | 119.9M | 144.5M |
| `mark` calls | 5 | 11 | 16 | 21 | 26 |

Each `mark` reaches the provider, which makes 4 Reflector calls per feed (`base`, `decimals`, `resolution`, `lastprice`) over 2 feeds. A full-holdings execute makes 351–609 contract calls, each instantiating a VM.

**On protocol 30 the binding step is simulation, not apply.** Two measured facts are kept apart:
1. *Fails recording-mode simulation at protocol 30* (`d6-02-local-resources.json`): the CLI simulation and a separate RPC `simulateTransaction` both return `Error(Budget, ExceededLimit)`; nothing is sent.
2. *Needs more than 100% of a protocol-30 limit when applied* (`d6-02-local-resources-raised.json`): every 8-action shape above and the caps C1–C4 below.

| Plan at protocol 30 (vault holds XLM and AQUA plus a Blend supply) | Result | Memory when applied |
| --- | --- | --- |
| 1 one-hop swap, S1-1 | applies | 53.6% |
| 1 one-hop swap + Blend Supply (the D4-05 receipt's shape), S7-1 | applies | 55.5% |
| Blend Supply only; Withdraw + Supply | applies | 50–52% |
| 1 one-hop buy of a third asset, W3 | fails simulation | 63.2% (diagnostic) |
| 2 or more one-hop swaps; any two-hop swap; any split swap; Withdraw + any swap + Supply; two-hop + Supply | fails simulation | n/a |
| Every candidate keeper cap C1–C4 (5 one-hop; 3 two-hop; Withdraw + 3 one-hop + Supply; Withdraw + 2 two-hop + Supply) | fails simulation | 113–123% (diagnostic) |
| 1 one-hop buy of XLM while holding 3, 4 or 5 non-USDC assets (ETH, BTC, USTRY added by direct SAC transfer), T3–T5 | fails simulation | n/a |

W3 fails simulation, yet the same execute applies at 63.2% under the diagnostic profile. Simulated instructions are within 4–6% of applied instructions for every plan, so CPU is not the gap; RPC 30.0.0 reports no simulation memory. If simulation memory is a constant multiple of applied memory, S7-1 (passes, 55.5% applied) and W3 (fails, 63.2% applied) put it at about 1.6–1.8×. Applied memory at 3 and 4 held is 79% and 95% (W4, W5): the ceiling is in the simulate-then-send flow.

**Local and live differ by one Blend action.** Locally at 2 held, one swap + Supply passes (S7-1). On live testnet at 2 held, one swap + Supply or Withdraw fails. The testnet Reflector holds more assets and history, and its `mark` simulates 5.8% higher. For planning, the live result governs: **one swap alone per execute** at 2 held.

**Operational consequence for the keeper (D6-01 row 6h).** In the retained runs, simulation is stricter than apply. No retained receipt shows a plan that passed simulation and then failed when applied. The keeper treats a budget failure in simulation as "split the plan".

**Recommended contract change.** Value each asset once per execute: cache the marks, and read each Reflector feed's `base`, `decimals` and `resolution` once or check them when the provider is configured (`etesia-vault/contracts/pricing/src/sources.rs:22-36` reads them on every call). An execute makes 26–35 `mark` calls where 6 assets need at most 6, so the change would remove most of the 208–280 Reflector calls (8 per `mark`).

**Fees.** Executes that created no new entries were charged 0.004–0.028 XLM (44,604–283,801 stroops). An execute that first created a balance entry or a Blend position cost 0.024–0.056 XLM, rent included. The local `fee_write1_kb` differs from live testnet (437), and mainnet write fees are dynamic.

## Finding 2: mainnet Soroswap liquidity covers only XLM and USTRY

The vault accepts only `Hop::Soroswap`; any other hop fails `Unauthorized`. On 2026-11-11 the universe pairs on Soroswap were (collector run above):

| Pair (Soroswap pool) | Reserves | Side USD | 24 h volume | 2026-09-17 side USD |
| --- | --- | --- | --- | --- |
| XLM/USDC `CAM7DY53…OABP` (the D3-verified pair) | 367,246.84 XLM / 54,372.80 USDC; implied 0.148055, −3 bps against CoinGecko | 54.4k / 54.3k | $3,525 (176 swaps) | 59.8k |
| USTRY/USDC `CDIXSYDR…DCCU` | 43,057.23 USTRY / 46,422.60 USDC; 1.0782 USDC per USTRY | 46.4k / 46.3k | $0 (1 swap) | 46.2k |
| XLM/AQUA `CDRS7NJP…LEVKI` | 41.25 XLM / 24,610.14 AQUA | $6 / $6 | $0 | $8 |
| AQUA/USDC `CB3CBFWC…CI4L63` | 1,640.45 AQUA / 0.40 USDC | about $0 | $0 | about $0 |
| BTC/USDC, ETH/USDC, BTC/XLM, ETH/XLM | **no Soroswap pool** | n/a | n/a | n/a |

The same assets are liquid on venues the vault does not accept: Aquarius XLM/USDC at $1.54M a side, USTRY/USDC $1.10M, AQUA/USDC $209k, ETH/USDC $63k, BTC/ETH $188k; SDEX AMMs XLM/USDC $2.2M, XLM/BTC $59k, BTC/USDC $43k.

**Consequence: with the Soroswap-only route policy, BTC, ETH and AQUA legs cannot be reached on mainnet.** The two options, accepting Aqua or SDEX hops (a contract change) or holding zero target weight in those assets, are in the [D6-05 checklist](d6-05-mainnet-readiness-checklist.md#findings-from-d6-02). limits-v1 calibrates XLM and USTRY legs.

## Proposed limits-v1 parameters (mainnet)

Contract bounds (`etesia-vault/contracts/vault/src/lib.rs:125-137`): `max_slippage_bps` ≤ 100, `max_leg_bps` ≤ 2,000, `max_turnover_bps` ≤ 10,000, `max_loss_bps` ≤ 200, `max_price_age` ≤ 3,600 s, `cooldown` ≥ 1. The turnover and loss window is 17,280 ledgers (`etesia-vault/contracts/vault/src/execution.rs:371`). Testnet values are from the pre-reset manifest `etesia-vault/deploy/testnet/pre-2026-12-16-reset.json`.

| Parameter | Testnet flagship / guard | Mainnet v1 | Derived from | Rationale |
| --- | --- | --- | --- | --- |
| `max_slippage_bps` (onchain) | 100 / 100 | **100** | D4-05 admissibility rule (cap − 10 bps margin = 90-bps budget); 30 bps Soroswap fee per hop; v3 `protocol_bps` 10 (the testnet copy's `config()`); [D6-01](d6-01-stress-matrix.md) rows 2a (thin tRESERVE legs skipped pre-trade at 513–1,770 bps) and 2b (a forced submission reverts with `InsufficientOutput`) | The floor is the last guard against a compromised executor. At 80, one-hop impact room falls from 50 to 30 bps, which cuts every leg by about 40% |
| Keeper slippage budget per hop count (offchain) | 1 hop | 1 hop: fees 40 bps, impact + deviation ≤ **50 bps**. 2 hops: fees 70 bps, ≤ **20 bps**. **3 or more hops: never** (fees about 100 bps, above the 90-bps budget) | Fees above | One budget per hop count and venue fee |
| Pool depth cap per leg (offchain) | n/a | Leg ≤ **0.5% of the input-side reserve** per hop at Reflector parity; 0.41% at −10 bps deviation; 0.13% at −37 bps (the 5-min p95 XLM move). On 2026-11-11: XLM/USDC **$276** at parity, $221 at −10 bps, $73 at −37 bps; USTRY/USDC **$235**, $188, $62; two-hop USTRY→USDC→XLM **$51** | Constant product on the 2026-11-11 reserves. Per hop, output value / input value = 0.997·R / (R + 0.997·x) for a leg x against input-side depth R (USD); the leg is admissible while 1 − 0.999 · Π(ratio) · (1 + deviation) ≤ 90 bps. [D6-01](d6-01-stress-matrix.md) rows 2a and 2b show the keeper's pre-trade skip keeps an over-deep leg from reaching the floor | The keeper already solves sizes against live quotes (`etesia-quant/src/etesia_quant/planner.py`, `solve`). This row bounds what that solve can return |
| `max_leg_bps` | 2,000 / 500 | **1,000** | d1-v5 leg p99.9 5.2%, max 6.7%; d1-v6 about 10–13% (×2) | Covers the strategy's p99.9 leg and bounds compromised-executor loss to 1% × 10% = 10 bps of equity per leg. The depth cap binds first above about $2.8k of equity |
| `max_turnover_bps` (per 17,280 ledgers) | 10,000 / 300 | **3,000** steady state. The initial deployment (up to about 95%) needs a ramp over 4 or more windows, or a temporary 10,000 through the timelocked admin path | d1-v5 day p99.9 9.9%, max 12.1%; d1-v6 about 20–24% (×2) | Covers the d1-v6 extreme day |
| `max_loss_bps` (per window) | 200 / 50 | **50** | Worst admissible: 90 bps × 30% turnover = 27 bps | Loss counts only execution shortfall at one mark set; 50 sits above the worst admissible window |
| `cooldown` (ledgers) | 12 / 720 | **60** (about 5 min) | Reflector resolution 300 s; Finding 1 forces about one swap per execute | Lets pairs re-arbitrage and a new oracle round land between legs. At 720, a 10-leg rebalance would take about 10 hours |
| `max_price_age` (vault) | 3,600 / 300 | **600 s** | Testnet feed ages (p50 197 s, max 3,037 s); XLM 10-min move p99 98 bps, 1 h move p99 331 bps; [D6-01](d6-01-stress-matrix.md) row 3 (priced calls fail closed past `max_price_age`, no cached mark; onchain, a guard-vault swap simulated with 308 s old marks fails `vault:Pricing`) | Deposits and redeems use the same marks. 3,600 s lets flows trade against marks up to about 3% stale (1 h p99); 600 s bounds that to about 1%. The cost is fail-closed pricing: trading, deposits and redeems stop while the feed is older than 600 s (one 3,037 s gap in 7 testnet days) |
| Provider `max_age` (per feed) | 3,600 | **600 s** | As above | Keep the provider no looser than the vault |
| Trigger band (offchain, D4-07) | 200 bps | **200 bps** | A band trade costs 2% × 40–90 bps = 0.8–1.8 bps of equity; backtest `band_factor` 2 | No change |
| Plan shape (offchain) | planner `MAX_ACTIONS = 8` (`etesia-quant/src/etesia_quant/planner.py:16`), split on budget failure | With the current contract: **one one-hop swap per execute, alone**, and only while at most 2 non-USDC assets are held, XLM included. A Blend Supply or Withdraw goes in its own execute | Finding 1 (live testnet and local) | Built as quant `1e8d2aa`: any plan of two or more actions over the budget goes out one swap per execute, else Blend-only |
| Launch AUM (offchain, derived) | n/a | At one swap per execute and a 60-ledger cooldown, a $276 leg allows at most about $3.3k of trades an hour on XLM/USDC | Rows above | A consequence, not a parameter |

## Concentration review

The contract enforces Risk and XLM gross ≤ 40% of equity when borrowing is off (`check_positions`, `etesia-vault/contracts/vault/src/execution.rs:452`). The strategy caps each asset at 40% (XLM 37.5%, `etesia-quant/src/etesia_quant/numerics.py:12`), with a 95% gross trend cap and two 2.5% buffers.

**Result: the 40% per-asset cap and the 30% sizing budget stay unchanged.** The binding mainnet constraints are reachability (Finding 2), depth (about $46–54k a side) and the per-execute resource ceiling (Finding 1), not concentration. The 40% cap exists for exit liquidity, which AUM sizing and the leg caps above handle better. Example: exiting a 37.5% XLM position in a $50k vault ($18.75k) takes about 68 legs at the $276 depth cap. The `max_turnover_bps` of 3,000 binds first: 37.5% of equity is more than one window allows, so the exit takes at least 2 windows (more than about 24 hours). A faster exit needs the temporary 10,000 through the timelocked admin path.
