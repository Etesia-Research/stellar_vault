# D3 pricing provider

Current A48 qualification scope excludes SHX: XLM/AQUA/ETH/BTC and USTRY require sources, with USDC as settlement. The generic constructor capacity below is not an approved asset list. Exact AQUA feed identity is verified, but its measured Soroswap liquidity remains inadequate for execution; see [acceptance](d3-acceptance.md).

Implemented locally **2026-09-18**, on vault baseline `8aa9290`. See [acceptance](d3-acceptance.md) and the
[network/source manifest](evidence/d3-network-manifest.json). Production network,
limits and full-universe acceptance remain open. No live provider is activated.
The provider was later simplified to Reflector only, removing the Soroswap TWAP
circuit breaker; recorded D3 evidence predates that change.

## Contract and units

[`contracts/pricing`](../contracts/pricing/src/lib.rs) builds `etesia_pricing`.
Its constructor fixes the USDC contract and up to seven non-USDC source mappings.
There is no administrator, price setter, upgrade entrypoint or source-switching
method. Deploy a new provider and a new vault to change immutable mappings.
The deployment caller must verify the source manifest and provider configuration
hash before funding the vault; constructors do not prove issuer/wrapper parity.

| Entrypoint | Behavior |
| --- | --- |
| `config()` / `config_hash()` | Immutable configuration and SHA-256 of its canonical Soroban XDR |
| `mark(asset)` | Vault `Mark { price, timestamp }`; errors on unavailable input, with no last-good fallback |
| `diagnose(asset)` | Configured source, valid feed legs read so far, mark and error code; the mark stays zero unless evaluation fully succeeds |

Pricing is **Reflector only**. The provider is stateless: it stores only its
configuration and reads no pool, history or second source. `mark.price` is
Reflector USDC per whole asset at `10^12`; `timestamp` is the oldest feed leg.
USDC/USDC is one at ledger time. Unknown assets fail. The vault rejects an oracle
call failure, a nonpositive price, a future timestamp or age above `max_price_age`.

Each Reflector read validates `base`, `decimals`, public `resolution()` in seconds,
and the exact configured `lastprice` key. All legs must be present, positive,
nonfuture and within their own `max_age` (1 to 3,600 seconds). A direct feed must
have base `Stellar(USDC)` and needs no denominator. A conversion has one same-base
USDC leg, keyed by exact `Stellar(USDC)` or the explicit `Other("USDC")` reference.
The latter's settlement-token basis remains a manifest approval requirement.
Never assume USDC/USD is one. SDK 22 wire structs use a zero-or-one-element vector
for the optional leg; the constructor rejects multiple conversion legs, duplicate
or USDC source assets, feed decimals above 18 and zero resolution.

Normalization uses `floor((n/10^nd)/(d/10^dd)*10^12)` with U256 intermediates,
up to 18 decimals, and one final rounding. Overflow or a zero normalized result
fails closed.

## Failure and restoration

Configuration is in instance storage; TTL is extended to 120,960 ledgers when
below 60,480 on every read. This storage lifetime is independent of economic
freshness. After feed restoration, old timestamps still fail age checks. Keep
provider code, instance and upstream feed entries alive using normal network TTL
operations; the provider cannot restore an archived code/instance or feed.

Errors: `1 Config`, `2 Unknown`, `3 Invocation`, `4 Identity`, `5 Missing`,
`6 Invalid`, `7 Future`, `8 Stale`, `9 Overflow`. Diagnostics use zero for success.
A failed priced action rolls back; automatic refusal does not set/clear the
vault guardian pause bit. Pricing recovers permissionlessly once every feed leg is
valid again. D2 qualified basket exits and price-independent position recovery
retain their existing requirements.

## Running and reproducing

Use the pinned Rust/CLI tools in [local RPC instructions](local-rpc.md):

```sh
sh scripts/check.sh
# In tools/source-probe:
corepack pnpm install --frozen-lockfile
# From the repository root; unsigned mainnet research only:
node tools/source-probe/probe.cjs docs/evidence/new-probe.json
# Optional actual-size quotes; credential remains in the process environment:
python3 tools/source-probe/quotes.py docs/evidence/new-probe.json docs/evidence/new-quotes.json
# Start scripts/local-node.py with a fresh ignored directory in another terminal.
python3 scripts/d3-local-smoke.py
python3 scripts/d3-local-smoke.py --conversion
node tools/source-probe/local-resources.cjs artifacts/local/d3/manifest.json docs/evidence/new-local-resources.json
python3 scripts/d3-record-evidence.py
```

The mainnet probe derives SAC identities from issuer/code, records ledger and
WASM hashes, and downloads the checked upstream pair WASM under ignored local
artifacts. Its pool reads are liquidity research only; they do not feed pricing.
It only invokes read methods through unsigned
simulation; it has no signing/submission path. Primitive return-value decoding
avoids SDK 13 parsing of newer protocol-28 transaction metadata. The pinned
CLI 22 cannot decode the current Reflector v6 specification; local RPC uses an
explicit controlled Reflector ABI fixture.
No claim of running deployed Reflector v6 on the protocol-22 node is made.

## Activation procedure still required

Freeze the exact network, full source/asset mapping, quote paths and measured
limits; bind a fresh vault to the provider. The compiled 3,600-second age
ceilings are implementation limits, not approved production parameters. Check
that provider limits are no looser than the vault's reviewed limits.

Fixed addresses do not freeze upstream upgrades. Before activation, establish
monitoring for code hashes, metadata, admin changes and feed age; have the
guardian pause on an unexpected code identity and require
review plus D2's timelocked resume. This operational control is specified but not
implemented as a D4 service. Direct priced exits remain subject to D2 pause
semantics; monitoring is not an atomic onchain code-hash lock in SDK 22.

Reflector is the single price source. There is no second-source or TWAP check:
a stale-but-within-age or wrong Reflector price flows straight into vault
accounting, bounded only by per-leg `max_age` and the vault's `max_price_age`.
The measured stale-mark entry/exit profit in [acceptance](d3-acceptance.md) needs
an approved mitigation or economic risk decision before activation. No emergency
stale-price override is added.
