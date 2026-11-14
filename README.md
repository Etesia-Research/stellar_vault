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

## Testnet deployment (Tranche 2)

The flagship vault `CD73M44EYNZOX7YWPSBMITVPERWNA6ZHDLAZ2XSM4RQM6V4XT7GYXFQH`
and the guard vault `CCUHQJ3FSGC26UOCLN3EK2DUO725ZLBVRUF5B7N36W6MB3DM4JEOC6VO`
run on Stellar testnet with vault WASM `ac188cbf790c3fa3a4112fb4f2abe0dfd587303936e2c132f2df9eb062d41b47`.
Both are configured with USDC, XLM and tRESERVE (Etesia's testnet stand-in for
USTRY), swap through an Etesia-deployed copy of the Soroswap Aggregator v3 and
supply USDC to the Blend testnet pool `CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF`.

- [Deployment manifest](deploy/testnet/pre-2026-12-16-reset.json): contract ids, assets, roles and limits.
- [Receipt archive](docs/evidence/t2-index.md): the Tranche 2 testnet transactions with their envelope, result and meta XDR.
- [Operations](docs/operations.md): deployment, signer roles, role rotation, direct exits and recovery.

## Implementation

**Current universe (2026-09-18):** XLM/AQUA/ETH/BTC risk assets, USTRY reserve and USDC settlement. SHX is temporarily excluded. Local fixture scripts configure these six assets; the contracts retain generic immutable asset configuration. AQUA remains unqualified for activation because its measured Soroswap liquidity is inadequate. See [D3 acceptance](docs/d3-acceptance.md).

D2 vault/adapter and D3 pricing provider, implemented and locally verified
2026-09-18; D3 was simplified to Reflector only on 2026-09-23. Independent Soroban workspace for USDC custody and guarded valuation.
Pricing is Reflector only, with no second-source check; see the D3 provider.
See [acceptance and evidence](docs/acceptance.md), [ABI](docs/interfaces.md),
[direct exits/recovery](docs/operations.md) and [local RPC reproduction](docs/local-rpc.md).
See [D3 provider](docs/pricing.md) and [D3 evidence/gates](docs/d3-acceptance.md).

Rust 1.99.1, SDK/token SDK 29.0.1, Stellar CLI 28.1.0 and cargo-llvm-cov 0.9.1;
Rust 1.84.1 builds the vendored DeFindex parent WASM harness. Dependencies and that
harness are pinned; the adapter mirrors the DeFindex strategy trait and errors.
Run from this repository with those tools on PATH:

```sh
sh scripts/check.sh
python3 scripts/local-node.py
# In another terminal:
python3 scripts/local-smoke.py
python3 scripts/d4-record-fixtures.py
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
