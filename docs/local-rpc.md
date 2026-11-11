# Reproduce local RPC acceptance

Verified 2026-11-09 on Linux amd64 at protocol 30; the 2026-09-18 protocol-22 run is
reproducible from commit `b318771`. The network is isolated, uses public standalone
genesis funds and generated fixture tokens, and is not Stellar Testnet/Mainnet.

## Tools and build

Use Rust 1.99.1 with the components/target (`wasm32v1-none`) in
`rust-toolchain.toml`, Stellar CLI 28.1.0 and `cargo-llvm-cov` 0.9.1, plus Rust
1.84.1 with `wasm32-unknown-unknown` for the DeFindex parent harness.
`Cargo.lock` pins SDK/token SDK 29.0.1, transitive SDK macros 29.0.1 and host
29.0.1. The optional repository-local installation is selected with
`. scripts/env.sh`.

```sh
rustup toolchain install 1.84.1 --profile minimal -t wasm32-unknown-unknown
cargo install cargo-llvm-cov --version 0.9.1 --locked
sh scripts/check.sh
sh scripts/install-local-node.sh
```

The node installer verifies official package SHA-256 values before extracting
Core 30.0.0 (`0e49751aa9ddf892302cca02cea7f3f5d97df9f1`, released 2026-10-16) and
RPC 30.0.0-220 (released 2026-10-21), the first releases that run protocol 30.
The host needs libsqlite3, libpq, libc++, libc++abi and libunwind. The Core
package declares libc++ 20; the acceptance host ran it with Debian libpq5,
libc++1-19, libc++abi1-19, libunwind8 and libunwind-19, extracted locally under
`.tooling/stellar`, and every symbol resolved under `LD_BIND_NOW=1`.
`LD_LIBRARY_PATH` is set by the node script. Existing system libraries are also usable. No system service installation
or Docker is required. `ETESIA_CORE` and `ETESIA_RPC` may identify existing pinned
binaries instead.

`check.sh` builds and optimizes the pinned upstream DeFindex vault with Rust
1.84.1 first, then builds the Etesia WASMs with `stellar contract build --optimize`
and checks that the optimized vault stays under `contract_max_size_bytes`
(131,072). It runs fmt/clippy/native tests and enforces at least 90% production
line coverage. Deploy the optimized WASMs in `target/wasm32v1-none/release`,
whose actual host compatibility is tested by RPC.

## Fresh network and round trip

```sh
python3 scripts/local-node.py
# In another terminal, from the same repository and pinned tool environment:
python3 scripts/local-smoke.py
```

The node process stays in the foreground; Ctrl-C terminates its children. It
refuses to reset an existing database. Supply a fresh directory argument for a
second run, with the prior process stopped. It creates local CLI network/key
configuration, launches validator/history/RPC, upgrades to protocol 30 and applies the
checked-in resource profile. CLI 28 reads the repository's `.stellar` only as its
config home, so both scripts set `STELLAR_CONFIG_HOME`. Core 30 reads
`ARTIFICIALLY_SET_CLOSE_TIME_FOR_TESTING` in milliseconds; the node sets 1,000 for
one-second ledgers. Logs and keys remain ignored. Ports are bound on
loopback: RPC 8003, validator HTTP 11626, captive HTTP 11826, history 1570.

The profile is explicit in
[`local-network-settings.json`](../fixtures/local-network-settings.json): the
unmodified [official quickstart protocol-30 testnet profile](https://github.com/stellar/quickstart/blob/f7fe0705dd/local/core/etc/config-settings/p30/testnet.json)
of quickstart commit `f7fe0705dd` (2026-11-09): 400 million instructions and 40 MiB
memory per transaction, 131,072-byte code, 132,096-byte transactions, 200 read
entries, 200 write entries, 200,000 read bytes and 132,096 write bytes. As
quickstart does, the node first applies that commit's `settings_enable_upgrades.json`
([`local-network-enable-upgrades.json`](../fixtures/local-network-enable-upgrades.json)),
because the genesis limits reject the full profile's upload transaction. These are
recorded local compatibility settings, **not** a claim that an unspecified live
deployment uses them. Verify current protocol/resources before accepting any
external network.

The smoke script checks the passphrase before acting. It deploys six local
USDC/XLM/AQUA/ETH/BTC/USTRY fixtures and a fixture oracle; the Soroswap
Aggregator v3, router, factory and pair WASMs (mainnet code, checked against
[`fixtures/wasm/SHA256SUMS`](../fixtures/wasm/SHA256SUMS)) from `fixtures/wasm`; one
Etesia-made USDC/XLM pair seeded at the fixture mark; and a funded vault whose
`route_contracts` is that pair. It deposits 100 USDC; publishes a bound target;
swaps to XLM through v3 with the vault's Reflector floor as `minimum`, and asserts
that the vault's XLM balance equals the v3 `swap` event's `amount_out` and is at
least `minimum`; collects fees; sells that XLM in a permissionless unwind; and
redeems the holder's complete share balance. It records addresses, config, WASM
hashes, target/plan, transaction logs and final state under ignored
`artifacts/local/`. No pool or external price source is activated.

After a passing smoke, `python3 scripts/d4-record-fixtures.py` (node still
running) checks every receipt is `SUCCESS` and writes the
[D4 local record](evidence/d4-local-rpc.json) and, for the vault's deploy and
invocations, the base64 receipt and decoded (`xdrFormat: "json"`) transaction in
[`fixtures/d5-04-local/`](../fixtures/d5-04-local/): the D5-04 decoder fixtures.
The fixture tokens emit no events. The reviewed public record is in
[evidence](evidence/).

Stellar network fees are paid by external local genesis funding. The portfolio's
XLM buffer is not presented as a fee-payer reimbursement mechanism.

The A48 six-asset rerun is recorded in [reduced-universe receipts](evidence/shx-exclusion-local-rpc.json). The original seven-asset evidence remains a dated historical snapshot.

## D6-02 execute resource profile

`scripts/d6-local-resources.py` measures `execute` against the protocol-30 limits on a fresh node.
It needs the testnet Reflector and the testnet Blend v2 pool, backstop and Comet WASMs, fetched read-only:

```sh
dir=$(mktemp -d)
for name in reflector:CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63 pool:CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF backstop:CBDVWXT433PRVTUNM56C3JREF3HIZHRBA64NB2C3B2UNCKIS65ZYCLZA comet:CA5UTUUPHYL5K22UBRUVC37EARZUGYOSGK3IKIXG2JLCC5ZZLI4BDWDM; do
  stellar contract fetch --id "${name#*:}" --rpc-url https://soroban-testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015" -o "$dir/${name%%:*}.wasm"
done
python3 scripts/d6-local-resources.py "$dir/reflector.wasm" "$dir/pool.wasm" "$dir/backstop.wasm" "$dir/comet.wasm" .tooling/local-run
```

The fetched SHA-256 values were Reflector `25fc3060…1439`, pool `a41fc53d…350e`, backstop
`c1f4502a…fbc2` and Comet `8abc2891…1a36` (2026-11-11); the record lists all of them.

The last argument must be the data directory of the node that is running (for its
`resource-limits.json`); the script checks it against the node and refuses another one. For the
diagnostic run below that directory is `.tooling/raised`, not `.tooling/local-run`. The script
deploys SAC assets issued by the standalone genesis account, all 15 universe pairs, the pricing
provider over a local copy of the Reflector code with one USD price round and a USDC denominator leg,
a Blend pool in status 2 and the vault. It supplies to Blend, then buys one more asset per execute
until a single swap no longer fits: each valuation prices every asset held, so cost grows with
holdings. From that state it sends each plan family largest first until one applies, then the
proposed keeper caps. If a buy stopped fitting, it finally adds the remaining assets by direct SAC
transfer and tries one one-hop buy at each holdings count. Routes have at most 2 hops and at most 2
routes per swap: the keeper sends 1 hop and 1 route, and the mainnet API quote allows 2 hops. The
contract's maximum of 4 hops and 15 routes per swap is not measured and would cost more. Each plan
is also simulated through RPC; RPC 30.0.0 returns no simulation `cost`, so only simulated
instructions are recorded, and simulation memory is not observable. Every receipt's
`core_metrics` diagnostic events, call counts, declared resources and fees are in
[the D6-02 record](evidence/d6-02-local-resources.json). The live testnet limits and one simulated
testnet `mark` are in [d6-02-testnet-limits.json](evidence/d6-02-testnet-limits.json).

To see what a plan needs beyond those limits, start the node with the diagnostic profile
[`d6-02-raised-limits.json`](../fixtures/d6-02-raised-limits.json), which raises only the instruction
limits (transaction and ledger), the memory limit and the event-size limit: `python3 scripts/local-node.py .tooling/raised
fixtures/d6-02-raised-limits.json`. Then every plan applies, and the script writes
[the raised record](evidence/d6-02-local-resources-raised.json), with headroom still computed against
the protocol-30 profile. A plan whose simulation exceeds the budget is never sent; one that passes
simulation and then fails when applied is kept as a failed receipt. The record is rewritten after
every scenario, so a run that stops keeps its receipts so far.

## Tranche 1 demonstration

The [verification notebook](../notebooks/tranche-1-demo.ipynb) runs the existing
checks and local smoke scripts in presentation order, verifies RPC receipts and
WASM hashes, and demonstrates Reflector price refusal and the NAV test. Run its
preparation cells before recording; full coverage and fresh deployments exceed
the short video segment. It uses only Python's standard library plus the pinned
tools above, and starts/stops its own fresh isolated node. Fixture evidence does not close the [live acceptance gates](d3-acceptance.md).
