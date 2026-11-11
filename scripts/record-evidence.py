#!/usr/bin/env python3
"""Record public receipts and source/build hashes after successful local acceptance.

--testnet OUT HASH...: archive successful testnet transactions of the pre-reset deployment by hash instead.
TESTNET_RPC_URL overrides the SDF testnet RPC; only its scheme and host are recorded.
"""
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import sys
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/evidence'
OUT.mkdir(exist_ok=True)


def rpc(url, method, params):
    request = urllib.request.Request(url, data=json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': method, 'params': params}).encode(),
                                     headers={'Content-Type': 'application/json', 'User-Agent': 'etesia-record-evidence'})  # SDF RPC refuses urllib's
    return json.load(urllib.request.urlopen(request, timeout=10))['result']


if sys.argv[1:2] == ['--testnet']:
    if len(sys.argv) < 4:
        sys.exit('usage: record-evidence.py --testnet OUT.json HASH...')
    out, hashes = Path(sys.argv[2]), sys.argv[3:]
    if out.exists():
        sys.exit(f'{out} exists; each record is written once')
    path = ROOT / 'deploy/testnet/pre-2026-12-16-reset.json'
    raw = path.read_bytes()
    url = os.environ.get('TESTNET_RPC_URL') or 'https://soroban-testnet.stellar.org'
    passphrase = json.loads(raw)['passphrase']
    assert rpc(url, 'getNetwork', {})['passphrase'] == passphrase == 'Test SDF Network ; September 2015'
    transactions = []
    for tx_hash in hashes:
        receipt = rpc(url, 'getTransaction', {'hash': tx_hash})
        assert receipt['status'] == 'SUCCESS', (tx_hash, receipt['status'])
        transactions.append({'hash': tx_hash, 'status': receipt['status'], 'ledger': receipt['ledger'],
                             'close': datetime.fromtimestamp(int(receipt['createdAt']), UTC).isoformat(),
                             **{k: receipt.get(k) for k in ('envelopeXdr', 'resultXdr', 'resultMetaXdr', 'diagnosticEventsXdr')}})
    ledgers = [t['ledger'] for t in transactions]
    closes = sorted(t['close'] for t in transactions)
    record = {'label': 'pre-2026-12-16-reset testnet deployment (A76); archived XDR is the record after the reset',
              'passphrase': passphrase, 'endpoint': '{0.scheme}://{0.netloc}'.format(urllib.parse.urlsplit(url)),
              'manifest': {'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(raw).hexdigest()},
              'recorded_at': datetime.now(UTC).isoformat(), 'ledgers': {'first': min(ledgers), 'last': max(ledgers)},
              'closes': {'first': closes[0], 'last': closes[-1]}, 'transactions': transactions}
    out.write_text(json.dumps(record, indent=2) + '\n')
    print(f'Recorded {len(transactions)} successful testnet transactions, ledgers {min(ledgers)}-{max(ledgers)}, in {out}')
    sys.exit()

manifest = json.loads((ROOT / 'artifacts/local/manifest.json').read_text())
assert manifest['result'] == 'passed'
assert manifest['passphrase'] == 'Standalone Network ; September 2026'
for item in manifest['transactions']:
    request = urllib.request.Request(manifest['rpc'], data=json.dumps({
        'jsonrpc': '2.0', 'id': 1, 'method': 'getTransaction',
        'params': {'hash': item['hash']}}).encode(), headers={'Content-Type': 'application/json'})
    receipt = json.load(urllib.request.urlopen(request, timeout=10))['result']
    assert receipt['status'] == 'SUCCESS', receipt
    item.update(status=receipt['status'], ledger=receipt['ledger'])
for name, digest in manifest['wasm_sha256'].items():
    path = ROOT / 'target/wasm32-unknown-unknown/release' / name
    assert hashlib.sha256(path.read_bytes()).hexdigest() == digest
manifest.pop('transactions_log')
manifest['resource_profile_sha256'] = hashlib.sha256((ROOT / 'fixtures/local-network-settings.json').read_bytes()).hexdigest()
manifest['scope'] = 'Six local token fixtures, fixture oracle/router, no external pool or live protocol activation'
manifest['tools'] = {'rust': '1.84.1', 'stellar_cli': '22.8.2', 'sdk': '22.0.8', 'host': '22.1.3', 'core': '22.4.1 (89b9af01e705e076cdc607177d7bb953d36c8d97)', 'rpc': '22.1.5-125'}
manifest['dependency_sources'] = {'sep56': '265d64edc87627707941a31bd12798b7fdeb47d1', 'defindex': 'be878a9c1b0b176ec4351a85dad5557425c4a97c', 'blend_v2': 'ba22b487b2c5057a4ecc28b05b5193c28e4bd117', 'soroswap_aggregator': '84de10e0f8d26168b4a76f8c23b963e50917517c'}
manifest['supported_actions'] = {'spot': ['Swap', 'unwind', 'redeem_in_kind'], 'pool': [], 'borrowing': []}
manifest['inactive_dependencies'] = ['pool; no live Blend integration in this local deployment']
manifest['disabled_actions'] = ['Collateral', 'Release', 'Borrow']
manifest['upstream_defindex_wasm_sha256'] = hashlib.sha256((ROOT / 'vendor/defindex/target/wasm32-unknown-unknown/release/defindex_vault.optimized.wasm').read_bytes()).hexdigest()
manifest['source_revision'] = 'uncommitted initial D2 workspace; source hashes recorded separately'
(OUT / 'local-rpc.json').write_text(json.dumps(manifest, indent=2) + '\n')
coverage = json.loads((ROOT / 'coverage/coverage.json').read_text())['data'][0]
report = {'tool': 'cargo-llvm-cov 0.6.16 / Rust 1.84.1 llvm-tools', 'scope': 'Etesia production line coverage',
          'exclusions': ['vendor/', 'testutils.rs', 'tests.rs', 'types.rs (contracttype-generated wire bindings)', 'tools/ (local fixtures)'],
          'warning': 'LLVM reports five mismatched macro-generated functions; reported metric is line coverage, not function coverage.',
          'lines': coverage['totals']['lines'], 'files': {str(Path(f['filename']).relative_to(ROOT)): f['summary']['lines'] for f in coverage['files']}}
(OUT / 'coverage.json').write_text(json.dumps(report, indent=2) + '\n')
paths = [ROOT / n for n in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml')]
for directory in ('contracts', 'tools', 'vendor/defindex'):
    paths += [p for p in (ROOT / directory).rglob('*') if p.is_file() and 'target' not in p.parts and 'test_snapshots' not in p.parts and (p.suffix in ('.rs', '.toml', '.lock') or p.name == 'LICENSE')]
files = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
source = {'files': files, 'sha256': hashlib.sha256(json.dumps(files, sort_keys=True, separators=(',', ':')).encode()).hexdigest()}
(OUT / 'source-manifest.json').write_text(json.dumps(source, indent=2) + '\n')
print('Recorded', len(manifest['transactions']), 'successful local transactions; source hash', source['sha256'])
