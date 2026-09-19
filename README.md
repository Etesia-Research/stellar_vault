# Etesia vault

[![Production line coverage](docs/coverage.svg)](#test-coverage)

## License

Copyright (c) 2026 Etesia Research Inc. All rights reserved.
Etesia-owned materials are available for inspection of the implementation and
demonstrations under the [Proprietary Source Inspection License](LICENSE).
Running, building, testing, copying, modifying, distributing or deploying them
requires prior written authorization and a separate agreement signed by both
parties, subject to the license's platform, third-party and legal exceptions.
The reproduction instructions below are for authorized users.

Vendored DeFindex code retains its [GPL license](vendor/defindex/LICENSE).
The [adapter](contracts/adapter/Cargo.toml) directly depends on that code;
the proprietary notice does not resolve GPL obligations for combined or derived
works. Before publication or distribution, resolve that compatibility with legal
counsel and, if necessary, the upstream rights holders. Other dependencies retain
their own licenses. GitHub's public-repository viewing and forking rights remain
unaffected.

## Implementation

**Current universe (A48, 2026-09-18):** XLM/AQUA/ETH/BTC risk assets, USTRY reserve and USDC settlement. SHX is temporarily excluded. Local fixture scripts configure these six assets; the contracts retain generic immutable asset configuration. AQUA remains unqualified for activation because breaker depth is inadequate. See [D3 acceptance](docs/d3-acceptance.md).

D2 vault/adapter and D3 pricing provider, implemented and locally verified
2026-09-18. Independent Soroban workspace for USDC custody and guarded valuation.
See [acceptance and evidence](docs/acceptance.md), [ABI](docs/interfaces.md),
[direct exits/recovery](docs/operations.md) and [local RPC reproduction](docs/local-rpc.md).
See [D3 provider](docs/pricing.md) and [D3 evidence/gates](docs/d3-acceptance.md).
Full D3 source acceptance, D4/D7/D9 activation and audit acceptance remain separate.

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

## Test coverage

The badge shows production line coverage from the last successful
`sh scripts/check.sh` run. The check enforces a 90% minimum and refreshes
`docs/coverage.svg` from the generated `coverage/coverage.json`; include the
updated badge when committing code changes. This is a local check, not a CI feed.
Vendored code, test utilities, tests, type definitions and tools are excluded
using the existing coverage filter in [the check script](scripts/check.sh).
