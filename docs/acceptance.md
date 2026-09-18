# D2 acceptance evidence

**Locally implemented and verified, 2026-09-18; initial D2 publication.**
The approved D2 foundation is implemented in this independent Rust repository.
No Testnet/Mainnet activation, real oracle/venue integration or audit acceptance
is implied. The [approved scope](../../doc/d2-vault-implementation-plan.md),
[interfaces](interfaces.md), [operations](operations.md) and
[reproduction guide](local-rpc.md) define the qualifications below.

## Guard review corrections — 2026-09-18

Corrections on top of `ca95392` address three reproduced review
findings: excess or unconsumed debt-asset purchases during public recovery,
blocked incremental deleveraging outside normal risk limits, and purchases of
zero-target reward assets. The [interface](interfaces.md) and
[recovery runbook](operations.md) describe the enforced boundaries.

`sh scripts/check.sh` passed on the corrected source: fmt, strict clippy,
upstream/Etesia optimized WASM builds, **46 native tests** (43 vault and 3 adapter),
and **93.7279% production line coverage (1,853 / 1,977)**. Seven new regression
tests cover the three failures, actual-output overshoot, existing repayment
balances, partial repayment, rollback, recovery while paused, consecutive partial
deleveraging, worsening leverage/health, ordinary-plan limits and reward sales.
The three primary regressions failed before the guard changes and pass afterward.

A fresh isolated protocol-22 network passed the seven-asset RPC smoke with the
corrected WASM, and all 29 receipts were independently verified as `SUCCESS`.
The [guard-fix evidence](evidence/guard-fixes.json) records source/build hashes,
coverage, transaction hashes and results. Its source digest combines the unchanged
initial manifest entries with the three changed Rust inputs. The original evidence
below remains the initial publication snapshot; no real pool or live dependency
activation is established by either local run.

## Recorded checks

| Check | Result |
| --- | --- |
| `sh scripts/check.sh` | Passed: fmt, strict workspace/all-target clippy, pinned upstream/Etesia WASM builds and optimization, native tests, coverage threshold |
| Native workspace tests | 39 passed: 36 vault invariants plus 3 adapter/upstream tests |
| Production line coverage | **93.4896%: 1,795 / 1,920 lines**, pinned cargo-llvm-cov 0.6.16 |
| Fresh local node | `scripts/local-node.py` reproduced protocol upgrade, resource profile and healthy RPC from a new database |
| Seven-asset RPC smoke | Passed deposit → shares → target → guarded swap → fees → public unwind → complete holder redemption |
| RPC receipts | 29 deployment/funding/flow transactions independently returned `SUCCESS`; core flow at ledgers 602–610 |
| Quant handoff | Python 3.14.7: all **192 tests passed** and `uv build` succeeded; existing default/history golden outputs unchanged |
| Storage | TTL extension and restored snapshot preserve supply, HWM, nonce, balances, target and pending change; original allowance expiry remains effective |

The authoritative public records are [local deployment/receipts](evidence/local-rpc.json),
[line coverage](evidence/coverage.json), and [source hashes](evidence/source-manifest.json).
Use `python3 scripts/record-evidence.py` after a successful smoke run to regenerate
them and verify receipt status/build hashes. The initial source-tree digest is
`d91af877061e624465210a317e11aaf2d583193f7aab762dcacba0e17a9dee68`.
The source hashes preserve the tested snapshot; publication commits are recorded in the workspace wiki.

Coverage includes vault and adapter production modules. Exclusions are tests,
fixture tools, vendored dependencies and the `types.rs` contracttype-generated
wire declarations. Mixed production/binding modules remain counted. LLVM reports
five mismatched macro-generated functions; the required metric is line coverage,
not function coverage. Strict Etesia clippy passes without warnings. Building the
unchanged upstream DeFindex source emits its existing unused-variable warning.

## Acceptance mapping

| Step | Implemented evidence |
| --- | --- |
| D2-01 | Rust/SDK/CLI/source pins, immutable configuration, local identity manifest and exact ABI/auth/event matrix; live asset/pool/code identities remain activation inputs |
| D2-02 | Funded seed, SEP-41/56 signatures, bounded flows, preview parity, fee shares/HWM/dust, dormancy catch-up, donation/rounding protection, loss recovery and qualified stale-price basket |
| D2-03 | Typed target/plan hashes, Python/Rust golden vector, nonce/deadline/config/cooldown/aggregate guards, malicious route/reentry rollback, timelocked roles/fees, pause and TTL/restored-snapshot tests |
| D2-04 | Accrued supply/collateral/debt and reward fixtures; exact Blend nested auth/position deltas; borrowing-off rejection and synthetic round trip; actual pinned DeFindex parent WASM, fee layering, failure rollback and public-unwind recovery |
| D2-05 | Reproducible optimized WASM build and isolated RPC startup; seven local assets and complete deposit/guarded-action/fee/exit receipts; direct holder recovery runbook |

Generated cash-flow sequences use seed `0x455445534941` and 100 steps, checking
asset/share conservation. Additional tests cover expired/delegated authorization,
fee recipient value, management frequency, retained loss carryforward, overdue
catch-up, target drift/replay, unsafe recipients, false reward reports, pool
illiquidity/re-entry, shrinking reserve caps, borrower interest/liquidation, and
bounded reduction of overweight assets. Exact nested-auth tests use only the
required root authorization; the suite does not rely solely on `mock_all_auths`.

The DeFindex harness builds the upstream vault from unchanged pinned contract
source and executes that WASM. It invests through the adapter, observes net fees,
fails an illiquid exit without burning parent shares, restores cash through the
vault's permissionless unwind, then completes parent withdrawal. Token/router/pool
fixtures remain local; this is D2 upstream compatibility evidence, not D7 live
portfolio unwinding or D9 resource acceptance.

The final RPC vault is `CDJL4C3N6NZ7LKRBJ6GXKWACC6FUVXXJHXH4SHCX5JDUYKQICZ7MVAIR`.
It deposited 1,000,000,000 USDC atoms, minted 100,000,000,031,709 share atoms and
redeemed 999,999,997 USDC atoms after elapsed management dilution. The holder's
remaining shares are zero; locked seed and crystallized fee shares remain.
All seven asset identities are explicitly **local fixtures**, not inferred issuers.

## Selected configuration and rationale

These are implementation choices within A42, not additional user approvals or
calibrated production limits.

| Parameter | Value and rationale |
| --- | --- |
| Deposit/NAV unit | USDC atomic units, seven decimals |
| Shares | Twelve decimals; 100,000 share atoms per initial USDC atom |
| Permanent seed | 1,000 funded USDC and 1,000 funded shares; seed is fee-bearing and never redeemable |
| Fees | 100 bps/year management; 1,000 bps performance |
| Immutable fee ceilings | 200 bps management; 2,000 bps performance, below the singular dilution denominator and limiting governance changes |
| Timelock | 34,560 ledgers; two nominal days at five seconds/ledger, allowing holder/guardian response |
| Price age/divergence | Local 300 seconds / 100 bps; constructor ceilings 3,600 seconds / 500 bps |
| Per-leg/slippage | At most 20% NAV / 100 bps; local defaults use these ceilings |
| Aggregate turnover/loss | 100% / 2% NAV per 17,280-ledger window, shared by execution and unwind; consumed amounts survive deposits/withdrawals |
| Capital changes | Deposits add to the window baseline; USDC withdrawals subtract cash; basket withdrawals reduce it pro rata; donations enter the next window baseline |
| Borrowing | Immutable off for flagship; separate native fixture ceiling 50% debt/equity and at least 125% risk-weighted health |
| Yield | USDC non-collateral supply only, at most 10% of residual reserve; selected live pool remains unactivated |
| Rewards | Zero atomic-unit dust threshold; positive unclaimed entitlement blocks priced cash flows |
| Network fees | Externally funded local tests; portfolio XLM reimbursement unactivated |

The substantial funded seed and twelve-decimal shares avoid a nominal one-unit
bootstrap denominator. Rounding loss for a deposit is less than one share atom's
asset value (`NAV / supply`), initially 10^-12 USDC. Donations
can raise that value, so signed bounded entrypoints remain essential: the seed
is not a promise of safety at an arbitrarily manipulated unbounded quote. Before
any attacker-owned shares, reducing a one-USDC deposit below one share atom would
require NAV above `10^22` USDC atoms (10^15 USDC) against 10^15 locked share atoms.
Pre-initialization donations enter actual NAV, never mint a second seed, and
cannot reset HWM when ordinary holders exit. Direct share burns also retain HWM.

Management uses the approved prorated-NAV formula, not daily AUM history. On
unchanged NAV, annual collection charges exactly 1%; 365 daily settlements charge
`1 - (1 - .01/365)^365`, about 0.99503%, a difference below 0.5 basis points of NAV.
The regression checks that bound. Performance uses fee-share dilution and upward
HWM rounding, carries sub-share entitlement, and never ratchets HWM on a
zero-share crystallization. High-value arithmetic is checked; out-of-range inputs
or intermediates fail atomically rather than saturating into a financial result.

## Explicit qualifications and later gates

- SEP-56 draft method/event signatures match the pin, but maxima are executable
  USDC liquidity limits; `max_redeem` is not an unconditional whole-balance promise.
  NAV-dependent views can fail on unavailable claims/rewards or fee catch-up.
- D3 must replace the custom `mark(asset)` fixture boundary with approved-source
  freshness, divergence, units and bootstrap evidence. Fixture readiness is not
  TWAP/Reflector coverage.
- D4 must verify live asset/pool/router code identities and actual route auth trees,
  pool eligibility, net yield versus USTRY, liquidity/re-entry, keeper restart and
  evidenced lending accounting/replays. All pinned Soroswap route structures are
  accepted under common guards; the multi-hop/split test uses a hostile local
  router, not live Phoenix/Aqua/Comet liquidity.
- D7 must verify the actual deployed DeFindex version and fee stack, full real
  holdings unwind, downtime/illiquidity behavior and audit/remediation.
- D8/D9 still require venue selection and a separately authorized borrowing vault,
  real atomic/staged resource proof, liquidation/bad-debt and stressed exit policy.
  Current USDC exits conservatively require all debt/collateral cleared first.
- The local protocol-22 profile uses 128 KiB code size; the old 64 KiB profile is
  too small. CPU/memory remain 100 million instructions / 40 MiB. Seven-asset RPC
  execution passes under the recorded profile, not an unspecified live profile.
- Native restoration renews TTL on unchanged archived-entry payloads. A real
  network archival/restore drill, monitoring and externally verified dependency
  availability remain operational activation checks.

The original 18-person-day figure remains submission history. This completed local
foundation does not establish a remaining live-integration duration; D3 coverage,
actual protocol/resource tests and audit scope must be measured before scheduling
those gates. No delivery date or revised capacity estimate is invented.
