# D3 pricing provider

Implemented locally **2026-09-18**, on vault baseline `8aa9290`. See [acceptance](d3-acceptance.md) and the
[network/source manifest](evidence/d3-network-manifest.json). Production network,
limits and full-universe acceptance remain open. No live provider is activated.

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
| `observe(asset)` | Permissionless direct reserve read from that asset's fixed pool; returns recorded reserves, normalized price, ledger and timestamp |
| `mark(asset)` | D2-compatible `Mark`; errors on unavailable input, with no last-good or single-source fallback |
| `diagnose(asset)` | Bounded configured source identities, valid feed legs read so far, three eligible samples, partial normalized mark and error code; partial values are never executable marks |
| `upkeep()` | Extend configuration/sample TTL without changing economic timestamps or fabricating observations |

`mark.price` is Reflector USDC per whole asset at `10^12`; `reference` is the
three-sample Soroswap trapezoidal TWAP. `timestamp` is the oldest constituent.
`ready` is true only after all checks pass. USDC/USDC is one by denomination.
Unknown assets fail. The vault retains positivity, age, readiness and rounded-up
divergence checks, with **Reflector (`price`) as denominator**.

Each Reflector read validates `base`, `decimals`, public `resolution()` in seconds,
and the exact configured `lastprice` key. All legs must be present, positive,
nonfuture and within their own age limit. A direct USDC feed needs no denominator.
A conversion has one same-base USDC leg, keyed by exact `Stellar(USDC)` or the
explicit `Other("USDC")` reference. The latter's settlement-token basis remains a
manifest approval requirement. Never assume USDC/USD is one. SDK 22 wire structs
use zero-or-one-element vectors for optional feed/sample records; the constructor
rejects multiple conversion legs.

Normalization uses `floor((n/10^nd)/(d/10^dd)*10^12)` with U256 intermediates,
up to 18 decimals, and one final rounding. Pool observations similarly normalize
USDC reserve atoms / asset reserve atoms, checking token order, actual token
decimals, positive reserves and separate minimum reserves on both sides.
This is marginal reserve pricing, not a fill, route quote or last-trade time.
Only direct asset/USDC breaker pools are implemented; no inferred multi-hop
breaker conversion or automatic alternative pool selection is provided.

For increasing timestamps, TWAP is
`floor(((p0+p1)*(t1-t0)+(p1+p2)*(t2-t1))/(2*(t2-t0)))`.
The comparison is `ceil(abs(TWAP-price)*10000/price) <= divergence_bps`.
Only the combined mean is compared; individual deviations do not veto it.
No interpolation beyond the observed interval occurs. All multiplications have
bounded U256 widths; overflow or a zero normalized result fails closed.

## Collection, failure and restoration

Temporary storage holds three completed-ledger observations and at most one
pending observation per asset. The first eligible observation in a ledger is
retained; another call that ledger fails. A new pending observation does not
replace any of the three usable observations in its ledger. On a subsequent
ledger, it deterministically joins the completed history and the oldest sample
is dropped. Callers provide no pool, price or time.

Every accepted collection must advance time by at least `min_spacing` and ledger
sequence. A gap greater than `max_gap` discards the old window; three new samples
are required. Reads separately check every sample's age, consecutive gap and
ledger order, and require the newest completed sample to be no older than
`max_gap`. An overdue window is therefore already unusable before a new write
resets it in that ledger. `max_skew` bounds the difference between the oldest and newest of
all samples and feed legs. No per-observation price dispersion rule is added.

Configuration is in instance storage; samples are temporary and cannot be
archived/restored into usable price history. TTL is extended to 120,960 ledgers
when below 60,480 by collection/upkeep; config reads also extend instance TTL.
This storage lifetime is independent of economic freshness. After configuration
restoration, expired temporary history is absent and collection bootstraps again.
After feed restoration, old timestamps still fail age checks. Keep provider code,
instance and upstream source entries alive using normal network TTL operations;
`upkeep` cannot restore an already archived code/instance or upstream feed.

Errors: `1 Config`, `2 Unknown`, `3 Invocation`, `4 Identity`, `5 Missing`,
`6 Invalid`, `7 Future`, `8 Stale`, `9 Skew`, `10 Depth`, `11 Spacing`,
`12 Bootstrap`, `13 Divergence`, `14 Overflow`. Diagnostics use zero for success.
A failed priced action rolls back; automatic refusal does not set/clear the
vault guardian pause bit. D2 qualified basket exits and price-independent
position recovery retain their existing requirements.

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
artifacts for the RPC test. It only invokes read methods through unsigned
simulation; it has no signing/submission path. Primitive return-value decoding
avoids SDK 13 parsing of newer protocol-28 transaction metadata. The pinned
CLI 22 cannot decode the current Reflector v6 specification; local RPC uses an
explicit controlled Reflector ABI fixture and actual mainnet Soroswap pair WASM.
No claim of running deployed Reflector v6 on the protocol-22 node is made.

`scripts/d3-observe.py --network NETWORK --source IDENTITY --provider ADDRESS
--config-hash HASH` performs one signed collection pass using the reviewed
provider configuration. Schedule it externally with spacing and monitor failures;
sustained operations belong to D4. A failed call stops the pass and requires an
operator/runner retry. This runner was exercised only against the isolated node.

## Activation procedure still required

Freeze the exact network, full source/asset mapping, quote paths and measured
limits; bind a fresh vault to the provider. The compiled ceilings (3,600 seconds
for ages/skew and 500 bps divergence) are implementation limits, not approved
production parameters. Check that provider limits are no looser than the vault's
reviewed limits. Three samples must be collected before acquiring an asset.

Fixed addresses do not freeze upstream upgrades. Before activation, establish
monitoring for code hashes, metadata, admin changes, source age/depth and sample
collection; have the guardian pause on an unexpected code identity and require
review plus D2's timelocked resume. This operational control is specified but not
implemented as a D4 service. Direct priced exits remain subject to D2 pause
semantics; monitoring is not an atomic onchain code-hash lock in SDK 22.

Depth at sample endpoints does not prove sustained liquidity. Same-pool timing
can be manipulated, samples can be stopped, and both sources can lag together.
The measured stale-mark entry/exit profit in [acceptance](d3-acceptance.md) needs
an approved mitigation or economic risk decision before activation. No emergency
stale-price override is added.
