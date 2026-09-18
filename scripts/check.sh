#!/bin/sh
set -eu
# Run from the repository root with the pinned Rust and Stellar CLI on PATH.
cargo fmt --check
python3 scripts/d3-vectors.py
cargo build --manifest-path vendor/defindex/Cargo.toml -p defindex-vault --locked --release --target wasm32-unknown-unknown
stellar contract optimize --wasm vendor/defindex/target/wasm32-unknown-unknown/release/defindex_vault.wasm
cargo build --workspace --locked --release --target wasm32-unknown-unknown
for contract in etesia_vault etesia_defindex_adapter etesia_local_fixture etesia_pricing etesia_pricing_fixture; do
  stellar contract optimize --wasm "target/wasm32-unknown-unknown/release/$contract.wasm"
done
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
mkdir -p coverage
cargo llvm-cov --workspace --locked --ignore-filename-regex '(/vendor/|/testutils.rs|/tests.rs|/types.rs|/tools/)' --json --output-path coverage/coverage.json
cargo llvm-cov report --ignore-filename-regex '(/vendor/|/testutils.rs|/tests.rs|/types.rs|/tools/)' --summary-only
python3 - <<'PY'
import json
report = json.load(open('coverage/coverage.json'))
lines = report['data'][0]['totals']['lines']
print('Production line coverage:', lines)
assert lines['percent'] >= 90, lines
PY
