# Pinned upstream source

`defindex/` contains `common`, `strategies/core`, `vault`, Cargo workspace files and the standard GPL-3.0 license text
matching its workspace license declaration from
[DeFindex revision be878a9c1b0b176ec4351a85dad5557425c4a97c](https://github.com/defindex-io/stellar-contracts/tree/be878a9c1b0b176ec4351a85dad5557425c4a97c).
Only workspace membership and its lockfile resolution were reduced to those
packages. Contract source is unchanged. The adapter compiles the real trait;
tests execute the actual upstream vault WASM built by `scripts/check.sh`.

Upstream source/build dependencies are excluded from Etesia coverage. Generated
build artifacts are ignored. Live DeFindex deployment/version/fee compatibility
must be checked separately; this snapshot does not identify a live parent vault.
