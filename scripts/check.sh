#!/bin/sh
set -eu
# Run from the repository root with the pinned Rust and Stellar CLI on PATH.
for wasm in soroswap_aggregator_v3 soroswap_router soroswap_factory soroswap_pair; do
  if [ ! -f "fixtures/wasm/$wasm.wasm" ]; then
    echo "Missing fixtures/wasm/$wasm.wasm: run scripts/fetch-wasm.sh." >&2
    exit 1
  fi
done
(cd fixtures/wasm && sha256sum --quiet -c SHA256SUMS)
cargo fmt --check
python3 scripts/d3-vectors.py
RUSTUP_TOOLCHAIN=1.84.1 cargo build --manifest-path vendor/defindex/Cargo.toml -p defindex-vault --locked --release --target wasm32-unknown-unknown
stellar contract optimize --wasm vendor/defindex/target/wasm32-unknown-unknown/release/defindex_vault.wasm
stellar contract build --locked --optimize
# contract_max_size_bytes on testnet and mainnet.
size=$(wc -c < target/wasm32v1-none/release/etesia_vault.wasm)
echo "Optimized vault WASM: $size bytes"
test "$size" -lt 131072
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
mkdir -p coverage
cargo llvm-cov --workspace --locked --ignore-filename-regex '(/vendor/|/testutils.rs|/tests.rs|/soroswap_tests.rs|/types.rs|/tools/)' --json --output-path coverage/coverage.json
cargo llvm-cov report --ignore-filename-regex '(/vendor/|/testutils.rs|/tests.rs|/soroswap_tests.rs|/types.rs|/tools/)' --summary-only
python3 - <<'PY'
import json
from pathlib import Path
report = json.load(open('coverage/coverage.json'))
lines = report['data'][0]['totals']['lines']
print('Production line coverage:', lines)
assert lines['percent'] >= 90, lines
percent = f"{lines['percent']:.2f}%"
Path('docs/coverage.svg').write_text(f'''<svg xmlns="http://www.w3.org/2000/svg" width="162" height="20" role="img" aria-label="Production line coverage: {percent}">
  <title>Production line coverage: {percent}</title>
  <rect width="104" height="20" fill="#555"/>
  <rect x="104" width="58" height="20" fill="#4c1"/>
  <g fill="#fff" text-anchor="middle" font-family="Verdana, sans-serif" font-size="11">
    <text x="52" y="14">line coverage</text>
    <text x="133" y="14">{percent}</text>
  </g>
</svg>
''')
PY
