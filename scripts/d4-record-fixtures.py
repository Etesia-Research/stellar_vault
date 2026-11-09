#!/usr/bin/env python3
"""Record D4-01 local-RPC receipts and the vault's decoded meta (D5-04 fixtures) after a passing local smoke."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / 'fixtures/d5-04-local'
manifest = json.loads((ROOT / 'artifacts/local/manifest.json').read_text())
assert manifest['result'] == 'passed'
assert manifest['passphrase'] == 'Standalone Network ; September 2026'
assert manifest['protocol'] == 30


def rpc(method, **params):
    request = urllib.request.Request(manifest['rpc'], data=json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': method, 'params': params}).encode(), headers={'Content-Type': 'application/json'})
    return json.load(urllib.request.urlopen(request, timeout=10))['result']


def version(*args):
    env = dict(os.environ, LD_LIBRARY_PATH=str(ROOT / '.tooling/stellar/usr/lib/x86_64-linux-gnu'))
    output = subprocess.run(args, text=True, capture_output=True, check=True, env=env).stdout
    return next(line for line in output.splitlines() if not line.startswith('Warning'))


assert rpc('getNetwork')['protocolVersion'] == 30
FIXTURES.mkdir(exist_ok=True)
for old in FIXTURES.glob('*.json'):
    old.unlink()
for i, item in enumerate(manifest['transactions']):
    receipt = rpc('getTransaction', hash=item['hash'])
    assert receipt['status'] == 'SUCCESS', receipt
    item.update(status=receipt['status'], ledger=receipt['ledger'])
    decoded = rpc('getTransaction', hash=item['hash'], xdrFormat='json')
    # The vault's deploy and invocations carry the vault and share-token events that D5-04 decodes;
    # compact, because the decoded meta of one execute alone is about 200 KB indented.
    if manifest['vault'] in json.dumps(decoded):
        item['fixture'] = f"fixtures/d5-04-local/{i:02d}-{item['method']}.json"
        (ROOT / item['fixture']).write_text(json.dumps({'method': item['method'], 'hash': item['hash'], 'receipt': receipt, 'decoded': decoded}, separators=(',', ':')) + '\n')
for name, digest in manifest['wasm_sha256'].items():
    path = next(p for p in (ROOT / 'target/wasm32v1-none/release' / name, ROOT / 'fixtures/wasm' / name) if p.exists())
    assert hashlib.sha256(path.read_bytes()).hexdigest() == digest
manifest.pop('transactions_log')
manifest['resource_profile_sha256'] = {n: hashlib.sha256((ROOT / 'fixtures' / n).read_bytes()).hexdigest() for n in ('local-network-enable-upgrades.json', 'local-network-settings.json')}
manifest['scope'] = ('Six local token fixtures and a fixture oracle; mainnet Soroswap Aggregator v3, router, factory and pair code; '
                     'one Etesia-made pair; '
                     'no Blend pool or live protocol activation')
lock = (ROOT / 'Cargo.lock').read_text()
manifest['tools'] = {'rust': version('rustc', '--version'), 'stellar_cli': version('stellar', '--version'),
                     'sdk': re.search(r'name = "soroban-sdk"\nversion = "([^"]+)"', lock)[1],
                     'core': version(str(ROOT / '.tooling/stellar/usr/bin/stellar-core'), 'version'),
                     'rpc': version(str(ROOT / '.tooling/stellar/usr/bin/stellar-rpc'), 'version')}
(ROOT / 'docs/evidence/d4-local-rpc.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('Recorded', len(manifest['transactions']), 'successful local transactions and', len(list(FIXTURES.glob('*.json'))), 'D5-04 fixtures')
