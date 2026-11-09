#!/bin/sh
set -eu
# Linux amd64, official pinned Debian packages, extracted without a system install.
mkdir -p .tooling/downloads .tooling/stellar
curl -fsSL https://apt.stellar.org/pool/jammy/s/st/stellar-core_30.0.0-3642.0e49751aa.jammy_amd64.deb -o .tooling/downloads/stellar-core.deb
curl -fsSL https://apt.stellar.org/pool/focal/s/st/stellar-rpc_30.0.0-220_amd64.deb -o .tooling/downloads/stellar-rpc.deb
sha256sum -c <<'HASHES'
37faa5b5044a3b5b91cd68c00a25fb87c158be425e20245bd80ddfd0899dcca6  .tooling/downloads/stellar-core.deb
1cc1b01f6bc2778ec7615cc1cc070445520f45fdd6524edad920c7ce30009b1c  .tooling/downloads/stellar-rpc.deb
HASHES
dpkg-deb -x .tooling/downloads/stellar-core.deb .tooling/stellar
dpkg-deb -x .tooling/downloads/stellar-rpc.deb .tooling/stellar
# The host must provide libsqlite3, libpq, libc++, libc++abi and libunwind.
# See docs/local-rpc.md for the package variants used by the acceptance run.
