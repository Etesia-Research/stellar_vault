# Reproduce local RPC acceptance

Verified 2026-09-18 on Linux amd64. The network is isolated, uses public standalone
genesis funds and generated fixture tokens, and is not Stellar Testnet/Mainnet.

## Tools and build

Use Rust 1.84.1 with the components/target in `rust-toolchain.toml`, Stellar CLI
22.8.2 and `cargo-llvm-cov` 0.6.16. `Cargo.lock` pins SDK/token SDK 22.0.8,
transitive SDK macros 22.0.11 and host 22.1.3. The optional repository-local
installation is selected with `. scripts/env.sh`.

```sh
cargo install cargo-llvm-cov --version 0.6.16 --locked
sh scripts/check.sh
sh scripts/install-local-node.sh
```

The node installer verifies official package SHA-256 values before extracting
Core 22.4.1 (`89b9af01e705e076cdc607177d7bb953d36c8d97`) and RPC 22.1.5-125.
The host needs libsqlite3, libpq, libc++, libc++abi and libunwind. The acceptance
host used Debian libpq5, libc++1-19, libc++abi1-19, libunwind8 and libunwind-19,
extracted locally under `.tooling/stellar`; `LD_LIBRARY_PATH` is set by the node
script. Existing system libraries are also usable. No system service installation
or Docker is required. `ETESIA_CORE` and `ETESIA_RPC` may identify existing pinned
binaries instead.

`check.sh` builds the pinned upstream DeFindex vault first, optimizes all WASMs,
runs fmt/clippy/native tests and enforces at least 90% production line coverage.
Optimization is required: raw Rust 1.84 WASMs contain padded encodings that this
host rejects as reference-type features despite the target flags. Deploy the
`.optimized.wasm` artifacts, whose actual host compatibility is tested by RPC.

## Fresh network and round trip

```sh
python3 scripts/local-node.py
# In another terminal, from the same repository and pinned tool environment:
python3 scripts/local-smoke.py
```

The node process stays in the foreground; Ctrl-C terminates its children. It
refuses to reset an existing database. Supply a fresh directory argument for a
second run, with the prior process stopped. It creates local CLI network/key
configuration, launches validator/history/RPC, upgrades protocol and applies the
checked-in resource profile. Logs and keys remain ignored. Ports are bound on
loopback: RPC 8003, validator HTTP 11626, captive HTTP 11826, history 1570.

The profile is explicit in
[`local-network-settings.json`](../fixtures/local-network-settings.json):
protocol 22, 100 million instructions/transaction, 40 MiB memory, 40 read entries,
25 write entries, 200,000 read bytes, 132,096 write bytes. It derives from the
[official quickstart protocol-22 testnet profile](https://github.com/stellar/quickstart/blob/master/local/core/etc/config-settings/p22/testnet.json),
with WASM size raised to 128 KiB and upload bandwidth to 144 KiB (ledger 260 KiB).
The legacy 64 KiB code limit cannot load this vault. These are recorded local
compatibility settings, **not** a claim that an unspecified live deployment uses
them. Verify current protocol/resources before accepting any external network.

The smoke script checks the passphrase before acting. It deploys six local
USDC/XLM/AQUA/ETH/BTC/USTRY fixtures, a fixture oracle/router and funded vault;
deposits 100 USDC; publishes a bound target; swaps to XLM under vault guards;
collects fees; performs permissionless unwind; and redeems the holder's complete
share balance. It records addresses, config, WASM hashes, target/plan, transaction
logs and final state under ignored `artifacts/local/`. No pool or external price
source is activated. The reviewed public record is in [evidence](evidence/).

Stellar network fees are paid by external local genesis funding. The portfolio's
XLM buffer is not presented as a fee-payer reimbursement mechanism.

The A48 six-asset rerun is recorded in [reduced-universe receipts](evidence/shx-exclusion-local-rpc.json). The original seven-asset evidence remains a dated historical snapshot.

## Tranche 1 demonstration

The [verification notebook](../notebooks/tranche-1-demo.ipynb) runs the existing
checks and local smoke scripts in presentation order, verifies RPC receipts and
WASM hashes, and demonstrates the pricing deviation check and NAV test. Run its
preparation cells before recording; full coverage and fresh deployments exceed
the short video segment. It uses only Python's standard library plus the pinned
tools above, and starts/stops its own fresh isolated node. D3 additionally needs
the source probe's locked dependencies and pinned upstream pair WASM, as explained
in the notebook. Fixture evidence does not close the [live acceptance gates](d3-acceptance.md).
