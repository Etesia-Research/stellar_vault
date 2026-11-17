# D6-05 mainnet readiness checklist

The mainnet deployment checklist for the D6 measure "mainnet deployment checklist completed and reviewed". The product runs on Stellar testnet. Each row states what a mainnet deployment needs: **Done (testnet)** rows carry their evidence from the pre-reset testnet deployment, a local node or a Tranche 2 record; **Open (mainnet)** rows are required before mainnet activation and carry the next step. The Etesia team reviewed the checklist on 2026-11-17; the review is recorded below.

## Review and audit

| Field | Value |
| --- | --- |
| Reviewers | Etesia team |
| Review date (UTC) | 2026-11-17 |
| Outcome and comments | Accepted as reviewed, with the decisions below |
| Auditor | Etesia team |
| Audit scope | The vault and pricing contracts, the Soroswap Aggregator v3 call path, the keeper's key handling, the yield-limit rule, and the mainnet vault revision's venue hops and valuation fix |
| Audit window | Before mainnet activation |

**Decisions.** The review took these decisions:
- Outcome: the checklist is accepted as reviewed.
- Audit scope: the vault and pricing contracts, the Soroswap Aggregator v3 call path, the keeper's key handling, the yield-limit rule, and the venue hops and valuation fix of the mainnet vault revision.
- Audit window: before mainnet activation.
- `limits-v1`: mainnet takes the proposed column of [D6-02](d6-02-limits-v1.md#proposed-limits-v1-parameters-mainnet).
- Yield-limit rule: mainnet takes it ([Platform](#platform)).
- The three [D6-02 findings](#findings-from-d6-02): the mainnet vault revision accepts every venue Aggregator v3 executes from a contract, and takes the per-execute valuation fix.

## Aggregator

| Item | Status | Evidence or next step |
| --- | --- | --- |
| Soroswap Aggregator v3 at `CARVQXFP…BDDC4` configured and hash-pinned; mainnet `config()` reviewed | Open (mainnet) | The mainnet v3 WASM was fetched read-only (SHA-256 `437653282e9c…3190`). The testnet copy `CBE32GUT…V5TH` runs the same WASM; its code hash and `config()` (`protocol_bps` 10, `min_partner_bps` 25, not paused) are pinned in the pre-reset manifest `etesia-vault/deploy/testnet/pre-2026-12-16-reset.json`. The mainnet manifest needs the same entries |
| v3 audit status | Open (mainnet) | The published Soroswap aggregator audit predates v3. The v3 call path is in the Etesia audit scope (review decision) |
| Mainnet route mapping: aggregator distribution, Aqua pool-hash allowlist, Sushi/Comet hops; the vault revision that accepts them | Open (mainnet) | The Tranche 2 vault accepts Soroswap hops only, because testnet has only Soroswap pools. The mainnet revision accepts Aquarius, SushiSwap V3 and Comet hops as well ([findings](#findings-from-d6-02)). Multi-hop routes stay in the vault (up to 4 hops and 15 routes, `etesia-vault/contracts/vault/src/execution.rs:180,190`). The keeper quotes with a testnet constant (`etesia-quant/src/etesia_quant/routing.py:42`) |
| Accepted response to a Soroswap pause, upgrade or deprecation of v3 | Open (mainnet) | Vault config is immutable, so the response is a guardian pause, review and redeploy. The monitor detects a change through the code hash and `config()` pinned in the manifest ([D6-03](d6-03-alert-evidence.md#evidence-table) A7 and A8) |

## Platform

| Item | Status | Evidence or next step |
| --- | --- | --- |
| Toolchain | Done (testnet) | `soroban-sdk` 29.0.1, Rust 1.99.1, Stellar CLI 28.1.0, local node on protocol 30. Mainnet activation: next row |
| Mainnet protocol-30 activation ledger | Open (mainnet) | On 2026-11-10 mainnet was on protocol 29 (ledger 65,406,350), with the protocol-30 vote on 2026-11-23. After the upgrade, record the activation ledger (`etesia-vault/tools/source-probe/probe.cjs activation`, which adds `mainnet_activation` to the D4-02 evidence) before any mainnet deployment |
| Reflector re-verification on mainnet | Open (mainnet) | Testnet used Reflector v7, External feed `CCYOZJCO…RN63`, 300 s resolution. Mainnet runs Reflector v6 `CALI2BYU…LE6M` (19,169 B, `etesia-vault/docs/evidence/d3-network-manifest.json`). Re-read the mainnet feed, its resolution and the assets it prices before the mainnet manifest |
| Resource profile re-measured on mainnet | Open (mainnet) | The [D6-02](d6-02-limits-v1.md#finding-1-the-protocol-30-memory-ceiling) protocol-30 profile was measured with the testnet Reflector v7 code (20,417 B) and testnet per-transaction limits; mainnet runs v6. Re-measure the execute shapes on mainnet, against the mainnet limits |
| BLND reward handling | Open (mainnet) | Testnet avoids it with a Blend pool that has no USDC supply emission. Any pending BLND entitlement blocks priced calls (`etesia-vault/contracts/vault/src/pricing.rs:128-140`) |
| AQUA, USTRY and wrapper coverage | Open (mainnet) | Testnet has no AQUA or USTRY feed and no meaningful BTC/ETH liquidity. The mainnet universe needs price feeds and pool mappings |
| Vault yield-limit rule | Open (mainnet) | The yield limit is the most USDC the vault may hold in Blend, set from equity (`etesia-vault/contracts/vault/src/execution.rs:135`). On the deployed vault, a lone swap fails `Limit` when supply sits at the limit, because its fees lower equity and so the limit. The rule on vault `master` (`7e4934c`, merged in `401452d`; not deployed) lets an execute without a Supply end up to `max_slippage_bps` of the limit above it (1% on testnet); it is adopted for the testnet redeploy after the 2026-12-16 reset. Mainnet takes it (review decision): it is in the audit scope and the D6-02 resources are re-measured on it |

## Economics and risk

| Item | Status | Evidence or next step |
| --- | --- | --- |
| Net yield vs USTRY | Open (mainnet) | Tranche 2 measured the testnet Blend accrual only. Compare it with USTRY on mainnet data |
| Stale-mark risk | Open (mainnet) | Reflector is the only price source; the residual stale-mark arbitrage is tested (`etesia-vault/contracts/pricing/src/tests.rs:455`). The mainnet `max_price_age` is one of the `limits-v1` values (proposed 600 s) |
| Fee ceilings and recipient | Open (mainnet) | Pre-reset vaults use 100 bps management and 1,000 bps performance fees, paid to the admin (manifest) |

## Operations

| Item | Status | Evidence or next step |
| --- | --- | --- |
| Multisig custody for admin and guardian | Open (mainnet) | Testnet uses one operator holding separate admin and guardian keys, each account with its master key only ([D6-04](d6-04-runbook-drills.md#written-operations-items)) |
| Compromised guardian | Open (mainnet) | A compromised guardian can cancel every announcement, its own replacement included, and hold the vault paused; exits stay open (`etesia-vault/contracts/vault/src/governance.rs:60-70`; `lib.rs:49-52`). The remedy is a new deployment. Next: guardian multisig (row above) and the new-deployment path in the mainnet operations runbook |
| Executor key separated from the public API process | Open (mainnet) | On testnet the key is a Railway secret in the quant API process |
| No testnet signing key's account used on mainnet | Open (mainnet) | Use fresh mainnet keys and confirm that no testnet signing key's account (admin, executors, guardians, v3 treasury, realign, liquidity, the Railway secrets included) signs for anything on mainnet |
| Keeper's testnet-only liquidity, realign and top-up paths off mainnet | Open (mainnet) | These paths exist because nobody arbitrages the testnet pools: the keeper's top-up and realign (`ETESIA_LIQUIDITY_KEEPER=1`, `etesia-quant/src/etesia_quant/keeper.py:937`), the faucet refill (`ETESIA_BLEND_FAUCET_URL`, `liquidity.py:1072`) and the Builder's liquidity request (`ETESIA_LIQUIDITY_API_KEY`, Builder `app/api/quant/[...path]/route.ts:24`). Today all quant code loads testnet manifests only (`network.py:51`). A mainnet keeper ships without these paths or with them disabled, and the mainnet Builder without the liquidity key |
| Monitor check of pending timelocked changes | Open (mainnet) | The monitor has no `pending_change()` check (no match in `etesia-quant/src/etesia_quant/monitor.py`), so the guardian's review of an announced change is manual. Add the check or keep the manual review in the mainnet runbook |
| Real-network archival drill | Open (mainnet) | The archival and restore drill ran on a local node only ([D6-04](d6-04-runbook-drills.md#archival-and-restore-drill-row-10) row 10). Repeat it on a live network before activation |
| Monitoring | Done (testnet) | The monitor runs in the quant service every 300 s over both testnet vaults (flagship `CD73M44E…XFQH`, guard `CCUHQJ3F…C6VO`) and alerts to Telegram; a healthchecks.io dead-man check runs outside the service. Real and induced alerts with their open and resolve times: [D6-03](d6-03-alert-evidence.md#evidence-table) |
| `limits-v1` | Done (testnet) | [D6-02 limits-v1](d6-02-limits-v1.md): every parameter with its derivation, a protocol-30 resource profile and a concentration review. Accepted as the D6-02 proposal, with the 40% per-asset cap and the 30% sizing budget unchanged |
| Mainnet values of the `limits-v1` parameters | Open (mainnet) | Set at the review to the [proposed column](d6-02-limits-v1.md#proposed-limits-v1-parameters-mainnet). Read before activation: the mainnet per-transaction limits, the mainnet v3 `protocol_bps` and the mainnet Reflector feed ages, for which testnet values stand in |
| Runbooks: pause, Resume, recovery, exit, keeper restart | Done (testnet) | [D6-04 drills](d6-04-runbook-drills.md#drill-table) on the pre-reset deployment, receipts archived: guardian `pause()`; the timelocked Resume, announced and applied (`6f4a0e63…d2aa`, ledger 5,709,899); permissionless `unwind` and `recover_supply` while paused; a reconciliation-mismatch halt and `keeper clear`; a basket exit by an Etesia test key with the keeper and indexer off; the indexer restart and backfill; the keeper kill and restart. Runbook text: [operations](https://github.com/Etesia-Research/stellar_vault/blob/master/docs/operations.md) |
| Stress results | Done (testnet) | [D6-01 stress matrix](d6-01-stress-matrix.md): volatility, liquidity gaps, oracle staleness, keeper and indexer downtime and transaction failures, each with its documented result; 13 invalid executions rejected onchain by the guard vault (row 10) |
| Provider review: what the immutable oracle prices | Open (mainnet) | Input recorded in the manifest `provider` entry |

## Findings from D6-02

These three findings come from [D6-02 limits-v1](d6-02-limits-v1.md). Each is resolved before mainnet; the review chose the option for each.

| Item | Status | Evidence | Decision |
| --- | --- | --- | --- |
| Soroswap-only routes cannot reach BTC, ETH or AQUA on mainnet | Open (mainnet) | On 2026-11-11 mainnet Soroswap had no BTC or ETH pool and about $6 a side in XLM/AQUA; only XLM/USDC (about $54k a side) and USTRY/USDC (about $46k) are usable. The vault accepts Soroswap hops only. [Finding 2](d6-02-limits-v1.md#finding-2-mainnet-soroswap-liquidity-covers-only-xlm-and-ustry) | The mainnet vault revision accepts every venue Aggregator v3 executes from a contract: Aquarius, SushiSwap V3 and Comet next to Soroswap, each hop against a pool allowlist (a contract change; see the route-mapping row above). SDEX is a classic path payment and cannot run inside a contract call. Not taken: zero target weight in BTC, ETH and AQUA |
| The protocol-30 memory ceiling blocks a six-asset vault | Open (mainnet) | With 2 non-USDC assets held (XLM counts), one swap fits per execute if it leaves at most 2 held (buying a third, W3, fails); live testnet confirmed it on 2026-11-11 (swap alone 87.9M instructions, Supply alone 82.0M, combined `Error(Budget, ExceededLimit)`), and the keeper now sends one swap per execute. With 3 or more held, even one swap fails simulation on a local protocol-30 node. [Finding 1](d6-02-limits-v1.md#finding-1-the-protocol-30-memory-ceiling); [D6-01](d6-01-stress-matrix.md) row 6h | Take the valuation fix below. Not taken: a launch with at most 2 non-USDC assets |
| Per-execute valuation fix as a later contract release | Open (mainnet) | An execute makes 26–35 `mark` calls, each with 8 Reflector calls. Proposed fix: value each asset once per execute and check feed metadata at provider configuration | Built into the mainnet vault and pricing revision (new deployment, audit scope); the resource profile is re-measured on it |
