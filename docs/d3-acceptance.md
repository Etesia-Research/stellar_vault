# D3 implementation and acceptance evidence

**Current scope — A48, 2026-09-18:** SHX is temporarily excluded from the vault and new simulated targets/reports. Qualification now covers XLM/AQUA/ETH/BTC, USTRY and USDC. Original seven-asset evidence below remains a historical snapshot; it does not describe the revised fixture configuration.

**AQUA follow-up:** [unsigned calls at ledger 64494947](evidence/d3-aqua-followup.json), collected 19:41:57 UTC, confirm the exact AQUA SAC, Reflector USDC feed, factory pair, token ordering and seven-decimal units. The 19:40 feed mark was 0.00034565733339 USDC/AQUA. Reserves were **1,313.7990305 AQUA and 0.4494706 USDC**. Feed coverage is present; breaker liquidity still fails qualification. No transaction was signed, and no depth limit was relaxed.

Reviewed **2026-09-18**. Local implementation is complete for the selected direct
pool mechanism; full D3 acceptance remains blocked. Baseline vault revision:
`8aa929092435ab82fccb8ae24f00bba9e4cc1387`. The
[source manifest](evidence/d3-source-manifest.json) identifies the exact tested files
from the implementation snapshot captured before the initial D3 commit.
See [provider interface/operations](pricing.md) and the
[research network manifest](evidence/d3-network-manifest.json).

## Reduced-universe verification

The A48 rerun passed `sh scripts/check.sh` (63 native tests, 93.99% production line coverage). [Fresh isolated-node receipts](evidence/shx-exclusion-local-rpc.json) record six fixture assets, five provider mappings, 26 successful D2 receipts, 69 direct D3 receipts and 32 conversion D3 receipts. Both D3 scenarios produced 1,005 USDC NAV after five donations, blocked invalid-feed redemption without state changes and redeemed 99.9999998 USDC after recovery. The evidence records current script/build hashes; it establishes local behavior only.

## Local verification

- `sh scripts/check.sh`: pinned fmt, strict clippy, optimized WASMs
  and **63 passing native tests**, **93.99% production line coverage** (2,204/2,345 lines). See [coverage](evidence/d3-coverage.json) for the
  final counts. Existing D2 accounting, fee, lending, conservative swap minima,
  rollback and qualified recovery regressions remain in the suite.
- Nine independent Python `Fraction` vectors share CSV inputs/expected results
  with Rust: mixed decimals, non-unit USDC/USD, USTRY above par, equal/unequal
  intervals, dust and U256-sized intermediates. Overflow/invalid values and
  rounded-up divergence boundaries are tested separately.
- Provider tests cover mean-only offsetting deviations, pending-ledger exclusion,
  repeated observations, spacing/gaps, source age/future/missing/negative records,
  conversion denominator failure, source metadata, pool token order/depth,
  configuration restoration with lost temporary history, permissionless recovery,
  guardian-pause independence and price-free basket exits. Failed deposit/redeem
  preserve state, holdings, holder balance and emitted events.
- [Direct local RPC](evidence/d3-local-rpc.json): seven fixture assets, six actual
  deployed upstream Soroswap pair WASMs, controlled feed fixture, immutable provider
  and fresh vault. Pair initialization, funding, `sync` and reserve reads were
  exercised with fixture factory/token identities; actual pair trading was not.
  81 successful receipts. NAV was 1,006 USDC after six one-token
  donations; a 100-USDC deposit minted fee-adjusted shares, invalidating a required
  feed blocked redemption without changing state, and restored pricing allowed
  redemption of 99.9999998 USDC. The two-atom difference follows fee/rounding rules.
- [Conversion local RPC](evidence/d3-conversion-local-rpc.json): another provider
  and vault with all six assets reading an additional same-base USDC denominator.
  37 successful receipts; the same round trip passed. This deliberately exercises
  extra conversion reads; fixture USDC/USDC legs do not prove actual USD feed basis.

The first full-universe run exhausted the unchanged 40 MiB local memory limit.
Reducing the new provider/fixture linker stack to 64 KiB resolved it; the existing
vault and network settings were retained. The deployed Reflector WASM has one
initial memory page, whereas the initial local Rust fixture reserved 17 pages.
The current provider/fixture use two pages. These are recorded implementation
measurements, not a target-network resource guarantee.

| Local unsigned simulation | Instruction budget including RPC margin | Read bytes | Read entries |
| --- | ---: | ---: | ---: |
| Six direct marks plus USDC NAV | 51,538,894 | 158,532 | 33 |
| Six conversion marks plus USDC NAV | 58,843,756 | 160,608 | 34 |
| One direct provider mark | 8,926,966 | 41,136 | 6 |
| One conversion provider mark | 10,277,483 | 43,212 | 7 |

See [direct resources](evidence/d3-local-resources.json) and
[conversion resources](evidence/d3-conversion-resources.json). Local limits were
100 million instructions, 200,000 read bytes, 40 read entries and 40 MiB memory.
The actual target network, distinct feed-contract layouts, lending exposure,
realized rewards and worst-case multi-action plans still require measurement.
RPC did not return exact memory consumption; success establishes only compliance
with the configured local limit. Live mainnet probes reported protocol 28; the
isolated node uses protocol 22.

## Real read-only source evidence

Four probes between 18:36 and 18:45 UTC observed two distinct five-minute feed
updates (18:35 and 18:40). Raw calls, ledgers and hashes are preserved in
[probe 1](evidence/d3-mainnet-1.json), [probe 2](evidence/d3-mainnet-2.json),
[probe 3](evidence/d3-mainnet-3.json), and [probe 4](evidence/d3-mainnet-4.json).
A successful call returning `null` is missing coverage, not a usable price.
Earlier USTRY pool errors were SDK metadata-decoding failures; the fourth probe
reads primitive results and verifies the pool. No transactions were submitted.

| Asset | What was verified | Remaining acceptance |
| --- | --- | --- |
| XLM | Native SAC, seven decimals, exact Stellar Reflector mark, direct USDC pair, token order/reserves/code hash | Calibration and sustained operation |
| AQUA | Exact issued SAC/feed/pair, seven decimals | Direct pair had only **0.4494706 USDC**; sufficient breaker depth is not established |
| USTRY | Exact stablebond SAC, USDC feed above par, direct USDC pair, seven decimals | Calibration and sustained operation |
| BTC / ETH | External reference-symbol USD marks | Exact Stellar wrappers, approved basis and breaker pools are unverified |
| SHX (excluded under A48) | Historical exact SAC and seven-decimal check | Outside current acceptance scope; earlier selected feed/direct pair coverage failed |
| USDC | Exact settlement SAC, seven decimals, separate `Other("USDC")/USD` reference | Conversion basis approval; USDC/USDC itself remains one |

Both feeds reported version 6, 14 decimals, 300-second public resolution and
86,400-second history retention, with admin identities and deployed WASM hash in
the manifest. This does not tie their current binary to the earlier source
research revision or prove upgrade governance. Required ABI methods were
simulated successfully; CLI 22 could not decode the v6 full specification.

Feed ages at the four probe starts were approximately 120, 197, 180 and 314
seconds. A 300-second local limit would refuse the last read even during this
ordinary sample. This short window cannot approve production age/skew limits.
A candidate 600-second age limit needs longer cadence/outage measurements and
stale-mark economics review; spacing, depth and divergence also remain unapproved.

[Actual-size HTTP quotes](evidence/d3-route-quotes-full.json) requested only
Soroswap at 100 and 1,000 USDC for XLM/AQUA/SHX/USTRY. Responses include identities,
input amounts, paths, protocol, price impact and output minima. They are quotes,
not fills, and were not synchronous with the earlier Reflector reads. The API-reported price impact was 88.26%/98.69% for AQUA and
76.57%/97.03% for SHX at the two sizes; these are severe liquidity failures for
normal guarded execution, not acceptable fills. Available routed quotes do not
repair a missing Reflector feed or qualify an inadequately deep direct breaker
pool. D4 must obtain a current trade-size quote, enforce the tighter quote-relative
and Reflector-derived minima, and satisfy the existing custody/plan guards.

## Residual timing and recovery risks

The deterministic test `feed_update_timing_exposes_residual_stale_mark_arbitrage`
repeats entry before and exit after a lagging mark updates, while both layers agree
at each valuation. On three 10-USDC entries, gains after configured fees/rounding
were **0.4477609, 0.4266306 and 0.4074008 USDC**. These are controlled arithmetic
scenarios, not observed market profits. Freshness plus mean-only comparison does
not remove stale-price arbitrage; activation needs a reviewed mitigation/risk
limit. No new delay, fee or flow restriction is silently introduced.

Recovery is permissionless when feeds and samples become valid again. A stopped
collector eventually blocks priced actions; temporary history loss requires
three completed-ledger samples. Guardian pause remains independent. Qualified
basket exits do not acquire a price dependency; external claims, debt/collateral
and reward restrictions remain in force.

## Gate status

| Task | Result |
| --- | --- |
| D3-01 | Manifest and probes implemented; coverage/network/calibration acceptance incomplete |
| D3-02 | Immutable provider, direct observer, normalization, mean-only breaker, TTL and diagnostics implemented and locally verified |
| D3-03 | Reflector-denominator vault integration and new-provider vault round trips verified locally; D2 guards retained |
| D3-04 | Actual pair WASM exercised locally; repeated unsigned feed/pool reads and trade-size quotes recorded; actual Reflector v6 local/target integration incomplete |
| D3-05 | Evidence review recorded; full-universe live acceptance blocked |

Do not activate or mark Tranche 1 accepted until source gaps, wrappers, adequate
pool depth, network/tool compatibility, calibrated limits, latency mitigation,
upstream upgrade monitoring and sustained collection/resource evidence are resolved.
