#!/bin/sh
set -eu
# Linux amd64, official pinned Debian packages, extracted without a system install.
mkdir -p .tooling/downloads .tooling/stellar
curl -fsSL https://apt.stellar.org/pool/stable/s/stellar-core/stellar-core_22.4.1-2521.89b9af01e.jammy_amd64.deb -o .tooling/downloads/stellar-core.deb
curl -fsSL https://apt.stellar.org/pool/stable/s/stellar-rpc/stellar-rpc_22.1.5-125_amd64.deb -o .tooling/downloads/stellar-rpc.deb
sha256sum -c <<'HASHES'
492fdf3c978e810d67a3a0675da4097c5bb161a08562c614266bd67730ce2003  .tooling/downloads/stellar-core.deb
9434c69988478ba77286c88daa504cf78bbcb90f2da90dc3d148e29714c5e6ca  .tooling/downloads/stellar-rpc.deb
HASHES
dpkg-deb -x .tooling/downloads/stellar-core.deb .tooling/stellar
dpkg-deb -x .tooling/downloads/stellar-rpc.deb .tooling/stellar
# The host must provide libsqlite3, libpq, libc++, libc++abi and libunwind.
# See docs/local-rpc.md for the package variants used by the acceptance run.
