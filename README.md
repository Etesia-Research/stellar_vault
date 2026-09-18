# Etesia vault

D2 local foundation, implemented and verified 2026-09-18. Independent Soroban
workspace for the USDC custody/share vault and one-parent DeFindex adapter.
See [acceptance and evidence](docs/acceptance.md), [ABI](docs/interfaces.md),
[direct exits/recovery](docs/operations.md) and [local RPC reproduction](docs/local-rpc.md).
Real D3/D4/D7/D9 activation and audit acceptance remain separate.

Rust 1.84.1, SDK/token SDK 22.0.8, Stellar CLI 22.8.2 and cargo-llvm-cov 0.6.16.
Dependencies and the actual vendored DeFindex trait/parent WASM harness are pinned.
Run from this repository with those tools on PATH:

```sh
sh scripts/check.sh
python3 scripts/local-node.py
# In another terminal:
python3 scripts/local-smoke.py
python3 scripts/record-evidence.py
```

For the optional repository-local tool installation, source `scripts/env.sh`.
Deploy optimized WASM only. Tool caches, build output, keys and node state are
ignored. Fixture assets/prices and public standalone keys are local-only.
