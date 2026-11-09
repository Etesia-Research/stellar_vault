#!/bin/sh
set -eu
# Fetches the mainnet Soroswap WASMs listed in fixtures/wasm/SHA256SUMS and checks them.
# Run from the repository root with the pinned Stellar CLI on PATH.
if [ -z "${MAINNET_RPC_URL:-}" ]; then
  echo "MAINNET_RPC_URL is unset: set it to a mainnet Stellar RPC URL (the CLI ships none)." >&2
  exit 1
fi
while read -r hash name; do
  stellar contract fetch --wasm-hash "$hash" -o "fixtures/wasm/$name" \
    --rpc-url "$MAINNET_RPC_URL" --network-passphrase 'Public Global Stellar Network ; September 2015'
done < fixtures/wasm/SHA256SUMS
cd fixtures/wasm && sha256sum -c SHA256SUMS
