# D2 interface contract

Implemented locally on 2026-09-18. The immutable source pins and validation scope
are recorded in [acceptance](acceptance.md). The generated contract specification
is embedded in each optimized WASM; Rust public types are in
[`types.rs`](../contracts/vault/src/types.rs).

## SEP-41 and SEP-56 draft 0.1.2

All amounts are signed `i128`, rejected when negative. USDC amounts have seven
decimals, shares twelve. Positive flow inputs are bounded by `MAX_AMOUNT`.
The SDK-only `Env` parameter is absent from the wire ABI.

| Methods | Authorization, result and rounding |
| --- | --- |
| `query_asset()` | USDC contract identity |
| `total_assets()`, `total_supply()` | Economic net assets; actually minted supply, respectively. The latter excludes unminted pending fees. |
| `convert_to_shares(assets)`, `convert_to_assets(shares)` | Simulate both pending fees; round down; no writes |
| `preview_deposit(assets)`, `preview_redeem(shares)` | Same fee-adjusted downward conversion |
| `preview_mint(shares)`, `preview_withdraw(assets)` | Same fee-adjusted conversion, rounded up |
| `deposit(assets, receiver, from, operator)` | Operator and asset provider authenticate; returns shares |
| `mint(shares, receiver, from, operator)` | Same auth; returns assets paid |
| `withdraw(assets, receiver, owner, operator)` | Operator authenticates; delegated operator consumes owner's share allowance; returns shares burned |
| `redeem(shares, receiver, owner, operator)` | Same auth; returns USDC paid |
| `max_deposit(receiver)`, `max_mint(receiver)` | Conservative implementation limits; zero for paused, invalid-price or locked-seed receiver states |
| `max_withdraw(owner)`, `max_redeem(owner)` | Executable idle-USDC limits, rounded down and capped by holder entitlement; zero while debt/collateral remains |
| `balance(id)`, `allowance(from, spender)`, `decimals()`, `name()`, `symbol()` | SEP-41 metadata/read methods; expired allowances read zero |
| `approve(from, spender, amount, expiration_ledger)` | Owner authenticates; expiration is inclusive |
| `transfer(from, to, amount)`, `burn(from, amount)` | Owner authenticates; seed cannot move or burn; direct burn settles fees first |
| `transfer_from(spender, from, to, amount)`, `burn_from(spender, from, amount)` | Spender authenticates and consumes unexpired allowance |

**Draft qualification:** `max_redeem` is liquidity-limited, rather than always
returning the entire balance as the draft describes. Economic claim and executable
liquidity are deliberately separate. Views requiring NAV can fail on missing
external claims/rewards or overdue fee catch-up; callers must handle contract
errors. A standard USDC exit never substitutes a basket or queue receipt.
Standard signatures and deposit/withdraw event fields match the pinned draft;
this is not an unqualified claim of every draft semantic.

Each flow also has a `_bounded` variant with the same first four arguments plus
`min_shares`, `max_assets`, `max_shares` or `min_assets`, respectively, followed by
`deadline` (UTC seconds). Prefer these for signed user transactions. Conversions
are estimates without liquidity guarantees. Standard methods use the same math.

`redeem_in_kind(shares, owner, receiver, minima, deadline)` authenticates the
owner, requires one minimum per canonically ordered asset, and verifies actual
zero supply/collateral/debt and no material unclaimed rewards. Management fees
settle first. With valid NAV, performance fees settle too; with unavailable NAV,
only the exiting fraction is waived, with no invented USDC valuation.

## Plans, roles and external boundaries

`publish_target(Target)` requires executor auth. Canonical SHA-256 input is the
Soroban XDR tuple `(Symbol("target_v1"), Target)`. `Target` binds network, vault,
epoch/expiry, model/dataset hashes, ordered identities/weights, residual-yield
fraction, equity, fee-adjusted supply and a raw spot/Blend-position holdings hash.
The reference hash excludes accrued interest so ledger-time accrual alone does
not invalidate sizing. Publication compares current balances; execution permits
at most 100 bps equity/supply drift. New holdings or larger drift require a fresh
publication. Golden vectors are shared with Python in
[`fixtures/d2-target-v1.json`](../fixtures/d2-target-v1.json).

`execute(Plan)` binds target hash/epoch, nonce, configuration version, deadline,
and up to eight typed actions. It requires executor auth and a strict reduction
in target distance. Supply includes only the approved settlement asset. Lending
cannot exceed the chosen fraction (at most 1,000 bps) of residual reserve.
Shrinking reserves block incompatible allocations until claims are withdrawn;
permissionless partial recovery remains available when a full correction cannot
fit one plan. Normal plans preserve already-established 2.5% liquid buffers;
initial buffer deficits can be built up in bounded steps. Recovery is independent
of strategy targets and can reduce an overweight position in bounded steps.

Only `Swap`, `Supply`, `Withdraw`, `Collateral`, `Release`, `Borrow`, `Repay`
are executable. Flagship borrowing/collateral use is immutable off. Total debt
is capped at 50% of equity at constructor maximum, implying at most 150% gross
assets; flagship risk/XLM exposure has a 40% per-asset ordinary-plan cap. Health
uses the pinned pool's risk factors and oracle, with at least 125% coverage.
Standard exits conservatively require **all** debt and collateral recovered;
proportional leveraged payout optimization is not activated.

Soroswap aggregator revision `84de10e0f8d26168b4a76f8c23b963e50917517c`
uses `swap_exact_tokens_for_tokens(token_in, token_out, amount_in,
amount_out_min, distribution, to, deadline) -> Vec<Vec<i128>>`. Distribution
contains the pinned numeric Soroswap/Phoenix/Aqua/Comet protocol enum, paths,
parts and optional Aqua pool hashes. Complete split paths are bounded to 15
routes and five assets per path. Final custody deltas, independent price minimum,
slippage and aggregate limits apply across the whole quote.

`RouteAuth` only supplies bounded nested authorization: exact input-token
transfers and independently bounded intermediate-token transfers from the vault and allowed swap functions at configured route contracts.
Intermediate balances must be unchanged at completion, so pre-existing holdings cannot subsidize a route. It is **not** an invocation API. There is no arbitrary execute, unlimited token
allowance, external payout recipient, sweep, code upgrade or direct-venue fallback.
Actual routed execution and code-hash/recipient-tree compatibility remain D4 gates.

Blend request IDs are 0 supply, 1 withdraw, 2 collateral supply, 3 collateral
release, 4 borrow and 5 repay. Exact pre/post positions and underlying transfers
are checked. Repayment requests themselves are capped at accrued debt because
the pinned implementation transfers the request amount before refunding excess.
Supply rounds down, debt up at the 12-decimal reserve-rate scalar. Rewards use
the pinned emission-index units; zero atomic units is the materiality threshold.
A positive unclaimed entitlement blocks priced flows until realized. Held rewards
are valued once through the same pricing boundary.

`pause()` is guardian-only. `announce(Change)` is admin-only, with one pending
change. `cancel(hash)` is guardian-only. `apply_change(hash)` is permissionless
after 34,560 ledgers, and new role holders must authenticate acceptance. Changes
are limited to bounded fee terms, roles and resume. Resume requires healthy NAV;
fee changes settle old terms first. Assets, dependency identities, risk limits,
borrowing mode and code are immutable; changes require a reviewed new deployment.

The executor can choose targets within hard limits; the contract does not prove
that it honestly ran the quant model. This trust remains explicit.

## DeFindex adapter

The adapter implements the actual vendored `DeFindexStrategyTrait` and
`StrategyError`, including `harvest(from, Option<Bytes>)`. Constructor arguments
are the underlying asset and `Vec<Val>[vault, parent]`. Binding is immutable.
`deposit(amount, from)` and `withdraw(amount, from, to)` authenticate that parent;
`balance(from)` returns its USDC economic claim after pending Etesia fees, never
raw shares. Withdraw burns rounded-up shares and verifies exact USDC receipt at
the parent-authorized recipient (upstream can send directly to its depositor).
`harvest` accepts only `None` and settles Etesia fees; it has no trading authority.

A shortage returns the upstream strategy error and rolls back the parent's burn.
Anyone can use the vault's bounded public unwind and retry. The adapter cannot
return a basket. DeFindex's separate vault/protocol fee layer is demonstrated by
the pinned upstream WASM harness; deployed fee terms remain a D7 verification gate.

## Event schema v1

Standard token events use `soroban-token-sdk`. Standard deposit topics are
`[deposit, operator, from, receiver]`; withdraw topics are
`[withdraw, operator, receiver, owner]`; data maps contain `assets` and `shares`.
Custom events have topics `[name, 1u32]` and these ordered payloads:

| Name | Payload |
| --- | --- |
| `init` | config, funded seed assets, locked shares |
| `fees` | management shares, performance shares, start/end timestamps, resulting HWM, fee terms |
| `waiver` | owner, exiting shares, pre-exit supply, retained HWM; deliberately no USDC amount |
| `basket` | owner, receiver, burned shares, ordered asset payouts |
| `target` | canonical hash, full target |
| `plan` | consumed nonce, target hash, pre/post NAV, turnover |
| `unwind` | consumed nonce, pre/post NAV |
| `position` | request type, asset, requested amount, pre/post raw Blend positions |
| `rewards` | reward asset, actual claimed atoms |
| `pause` | true |
| `announce` / `cancel` | full pending change / hash |
| `config` | incremented version, executed pending change |

Price/position views and public manifests supplement event indexing; interest
between transactions is read from accrued reserves. D5 must retain that distinction
from cash flows, share dilution, paper returns and externally paid network costs.
