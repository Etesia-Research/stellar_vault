#!/usr/bin/env python3
"""Run an isolated protocol-22 network. Only public standalone test keys are used."""
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
DATA = Path(sys.argv[1] if len(sys.argv) > 1 else '.tooling/local-run').resolve()
CORE = str(Path(os.environ.get('ETESIA_CORE', '.tooling/stellar/usr/bin/stellar-core')).resolve())
RPC = str(Path(os.environ.get('ETESIA_RPC', '.tooling/stellar/usr/bin/stellar-rpc')).resolve())
PASSPHRASE = 'Standalone Network ; September 2026'
os.environ.setdefault('LD_LIBRARY_PATH', str(ROOT / '.tooling/stellar/usr/lib/x86_64-linux-gnu'))
if (DATA / 'core.db').exists():
    raise SystemExit('Choose a fresh local data directory; existing ledgers are never reset.')
DATA.mkdir(parents=True, exist_ok=True)
(DATA / 'history').mkdir()
children = []


def run(args, **kwargs):
    return subprocess.run(args, text=True, capture_output=True, check=True, **kwargs).stdout.strip()


def core_http(path, **params):
    return urllib.request.urlopen('http://127.0.0.1:11626/' + path + '?' + urllib.parse.urlencode(params), timeout=5).read().decode()


def start(args, name):
    stream = (DATA / (name + '.log')).open('w')
    process = subprocess.Popen(args, stdout=stream, stderr=subprocess.STDOUT)
    stream.close()
    children.append(process)


def wait_for(check, timeout=180):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        for child in children:
            if child.poll() is not None:
                raise RuntimeError('Local process stopped; inspect logs in ' + str(DATA))
        try:
            if check():
                return
        except (OSError, ValueError, KeyError):
            pass
        time.sleep(1)
    raise TimeoutError('Local node readiness timeout; inspect logs in ' + str(DATA))


(DATA / 'core.cfg').write_text(f'''HTTP_PORT=11626
PUBLIC_HTTP_PORT=false
PEER_PORT=11625
NETWORK_PASSPHRASE="{PASSPHRASE}"
# Public standalone validator seed; never use on a funded external network.
NODE_SEED="SDQVDISRYN2JXBS7ICL7QJAEKB3HWBJFP2QECXG7GZICAHBK4UNJCWK2 self"
NODE_IS_VALIDATOR=true
DATABASE="sqlite3://{DATA}/core.db"
BUCKET_DIR_PATH="{DATA}/buckets"
LOG_FILE_PATH="{DATA}/core-internal.log"
ARTIFICIALLY_ACCELERATE_TIME_FOR_TESTING=true
ARTIFICIALLY_SET_CLOSE_TIME_FOR_TESTING=1
FAILURE_SAFETY=0
UNSAFE_QUORUM=true
[QUORUM_SET]
THRESHOLD_PERCENT=100
VALIDATORS=["$self"]
[HISTORY.local]
get="cp {DATA}/history/{{0}} {{1}}"
put="cp {{0}} {DATA}/history/{{1}}"
mkdir="mkdir -p {DATA}/history/{{0}}"
''')
(DATA / 'captive.cfg').write_text(f'''NETWORK_PASSPHRASE="{PASSPHRASE}"
HTTP_PORT=11826
PUBLIC_HTTP_PORT=false
PEER_PORT=11825
DATABASE="sqlite3://{DATA}/captive.db"
ARTIFICIALLY_ACCELERATE_TIME_FOR_TESTING=true
ENABLE_SOROBAN_DIAGNOSTIC_EVENTS=true
ENABLE_DIAGNOSTICS_FOR_TX_SUBMISSION=true
UNSAFE_QUORUM=true
FAILURE_SAFETY=0
[[VALIDATORS]]
NAME="local_core"
HOME_DOMAIN="core.local"
PUBLIC_KEY="GCTI6HMWRH2QGMFKWVU5M5ZSOTKL7P7JAHZDMJJBKDHGWTEC4CJ7O3DU"
ADDRESS="localhost:11625"
QUALITY="MEDIUM"
HISTORY="curl -sf http://localhost:1570/{{0}} -o {{1}}"
''')
(DATA / 'rpc.cfg').write_text(f'''ENDPOINT="127.0.0.1:8003"
NETWORK_PASSPHRASE="{PASSPHRASE}"
STELLAR_CORE_URL="http://localhost:11826"
CAPTIVE_CORE_CONFIG_PATH="{DATA}/captive.cfg"
CAPTIVE_CORE_STORAGE_PATH="{DATA}/captive"
STELLAR_CORE_BINARY_PATH="{CORE}"
HISTORY_ARCHIVE_URLS=["http://localhost:1570"]
DB_PATH="{DATA}/rpc.sqlite"
STELLAR_CAPTIVE_CORE_HTTP_PORT=11826
CHECKPOINT_FREQUENCY=8
''')
# Standalone genesis account is defined by SHA-256(network passphrase).
raw = bytes([144]) + hashlib.sha256(PASSPHRASE.encode()).digest()
seed = base64.b32encode(raw + binascii.crc_hqx(raw, 0).to_bytes(2, 'little')).decode()
identity = ROOT / '.stellar/identity/local-root.toml'
identity.parent.mkdir(parents=True, exist_ok=True)
identity.write_text('secret_key = ' + json.dumps(seed) + '\n')
identity.chmod(0o600)
root = run(['stellar', 'keys', 'address', 'local-root'])
run(['stellar', 'network', 'add', 'local', '--rpc-url', 'http://127.0.0.1:8003', '--network-passphrase', PASSPHRASE])
for command in ('new-db', 'new-hist', 'force-scp'):
    run([CORE, command, *(['local'] if command == 'new-hist' else []), '--conf', str(DATA / 'core.cfg')])
signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
try:
    start([CORE, 'run', '--conf', str(DATA / 'core.cfg')], 'core')
    wait_for(lambda: json.loads(core_http('info'))['info']['ledger']['num'] > 2)
    core_http('upgrades', mode='set', upgradetime='1970-01-01T00:00:00Z', protocolversion=22)
    wait_for(lambda: json.loads(core_http('info'))['info']['ledger']['version'] == 22)
    encoded = run(['stellar', 'xdr', 'encode', '--type', 'ConfigUpgradeSet', 'fixtures/local-network-settings.json'])
    upgrades = run([CORE, 'get-settings-upgrade-txs', root, '0', PASSPHRASE, '--xdr', encoded, '--signtxs'], input=seed + '\n').splitlines()
    assert len(upgrades) == 9, 'Unexpected pinned core upgrade output'
    for envelope in upgrades[:8:2]:
        result = json.loads(core_http('tx', blob=envelope))
        assert result['status'] == 'PENDING', result
        time.sleep(3)
    core_http('upgrades', mode='set', upgradetime='1970-01-01T00:00:00Z', configupgradesetkey=upgrades[-1])
    time.sleep(3)
    (DATA / 'resource-limits.json').write_text(core_http('sorobaninfo'))
    start([sys.executable, '-m', 'http.server', '1570', '--bind', '127.0.0.1', '--directory', str(DATA / 'history')], 'history')
    start([RPC, '--config-path', str(DATA / 'rpc.cfg')], 'rpc')
    def healthy():
        request = urllib.request.Request('http://127.0.0.1:8003', data=b'{"jsonrpc":"2.0","id":1,"method":"getHealth"}', headers={'Content-Type': 'application/json'})
        return json.load(urllib.request.urlopen(request, timeout=5)).get('result', {}).get('status') == 'healthy'
    wait_for(healthy)
    print('Local RPC ready at http://127.0.0.1:8003; run scripts/local-smoke.py in another terminal.', flush=True)
    while True:
        time.sleep(10)
        if any(p.poll() is not None for p in children):
            raise RuntimeError('A local node process stopped')
finally:
    for process in reversed(children):
        process.terminate()
    for process in reversed(children):
        process.wait(timeout=20)
