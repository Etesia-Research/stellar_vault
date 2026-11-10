#!/usr/bin/env python3
"""Testnet only: the D4-02 accounts, the realign trustline and the Etesia-deployed copy of Soroswap's mainnet v3 WASM.
D4-03 extends it with assets and vaults. Run from etesia-vault/: python3 scripts/testnet-deploy.py <accounts|v3>

No secret is read: `keys secret` is never run, and command() logs stdout and stderr to the gitignored
artifacts/local/commands.jsonl. Steps already done (identity, funding, trustline, deploy, extensions) are skipped,
and a step's transaction hashes are saved even when the CLI fails after signing, so a rerun checks those
transactions instead of sending new ones. The fixed salt also makes a second deploy of the copy impossible.
"""
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
os.environ.setdefault("STELLAR_CONFIG_HOME", str(ROOT / ".stellar"))
os.environ["PATH"] = f"{ROOT / '.tooling/bin'}{os.pathsep}{os.environ['PATH']}"
PASSPHRASE = "Test SDF Network ; September 2015"
INPUT = ROOT / "tools/source-probe/testnet.json"
EVIDENCE = ROOT / "docs/evidence/d4-testnet-probes.json"
LABEL = "testnet rehearsal evidence; Etesia-deployed copy of Soroswap's mainnet v3 WASM; no mainnet claim"
EPOCH = "pre-2026-12-16-reset"
OUT = ROOT / "artifacts/local"
OUT.mkdir(parents=True, exist_ok=True)
log = OUT / "commands.jsonl"
STATE = OUT / "testnet-deploy.json"  # tx hashes between runs; gitignored
ADMIN, TREASURY, REALIGN = "etesia-admin", "etesia-v3-treasury", "etesia-realign"
WASM = ROOT / "fixtures/wasm/soroswap_aggregator_v3.wasm"
ALIAS = "etesia-v3-copy"
LEDGERS_TO_EXTEND = 640_000  # about 37 days from 2026-11-11, past the 2026-12-16 reset
SALT = hashlib.sha256(b"etesia-v3-copy").hexdigest()  # a deterministic id: a repeated deploy fails in simulation
inputs = json.loads(INPUT.read_text())
state = json.loads(STATE.read_text()) if STATE.exists() else {}
for var in ("STELLAR_RPC_URL", "STELLAR_NETWORK_PASSPHRASE", "STELLAR_RPC_HEADERS", "STELLAR_NETWORK"):
    os.environ.pop(var, None)  # each would override --network testnet in the CLI


class CliError(RuntimeError):
    def __init__(self, stderr, hashes):
        super().__init__(stderr)
        self.hashes = hashes


def command(args):
    result = subprocess.run(["stellar", *args], text=True, capture_output=True)
    with log.open("a") as stream:
        stream.write(json.dumps({"args": args, "returncode": result.returncode,
                                 "stdout": result.stdout, "stderr": result.stderr}) + "\n")
    hashes = list(dict.fromkeys(re.findall(r"(?:Signing transaction|Transaction hash|Transaction Hash):\s*([0-9a-f]{64})", result.stderr)))
    if result.returncode:
        raise CliError(result.stderr, hashes)  # the CLI may fail after it signed and sent
    return result.stdout.strip(), hashes


def deploy(wasm, source, network, alias, **kwargs):
    args = ["contract", "deploy", "--wasm", str(wasm), "--optimize=false", "--salt", SALT, "--source-account", source,
            "--network", network, "--alias", alias, "--"]
    for key, value in kwargs.items():
        args += ["--" + key.replace("_", "-"), str(value)]
    stdout, hashes = command(args)
    return stdout.splitlines()[-1], hashes


def rpc(method, **params):
    # The testnet RPC answers 403 to urllib's default User-Agent.
    request = urllib.request.Request(inputs["rpc"], data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(),
                                     headers={"Content-Type": "application/json", "User-Agent": "etesia-testnet-deploy"})
    return json.load(urllib.request.urlopen(request, timeout=30))["result"]


def included(hashes):
    """{hash, ledger, created_at} of each transaction; None when one failed or never landed (after 60 s of polling)."""
    rows = []
    for h in hashes:
        for _ in range(12):
            r = rpc("getTransaction", hash=h)
            if r["status"] != "NOT_FOUND":
                break
            time.sleep(5)
        if r["status"] != "SUCCESS":
            print(h, r["status"], flush=True)
            return None
        rows.append({"hash": h, "ledger": r["ledger"], "created_at": datetime.fromtimestamp(int(r["createdAt"]), UTC).isoformat()})
    return rows


def save():
    STATE.write_text(json.dumps(state, indent=2) + "\n")


def step(key, run):
    """Send a step's transactions once. Its hashes are saved before inclusion is checked, also when the CLI fails,
    so a rerun checks them instead of sending again. A step whose transaction failed or never landed is forgotten
    and sent again on the next run."""
    if key not in state:
        try:
            state[key] = run()
        except CliError as error:
            if error.hashes:
                state[key] = {"hashes": error.hashes, "cli_error": str(error)[-2000:]}
                save()
            raise
        save()
    if "transactions" not in state[key]:
        assert state[key]["hashes"], f"{key}: no transaction hash in the CLI output; see {log}"
        rows = included(state[key]["hashes"])
        if rows is None:
            del state[key]
            save()
            sys.exit(f"{key}: a transaction failed or never landed; the step runs again on the next run")
        state[key]["transactions"] = rows
        save()
    return state[key]


def fetch(*args):
    return json.loads(command(["ledger", "entry", "fetch", *args, "--network", "testnet"])[0])["entries"]


def version(*args):
    return next(line for line in subprocess.run(args, text=True, capture_output=True, check=True).stdout.splitlines() if line)


def meta(captured_at, ledgers):
    git = lambda d: {"commit": version("git", "-C", d, "rev-parse", "HEAD"),
                     "dirty": bool(subprocess.run(["git", "-C", d, "status", "--porcelain"], text=True, capture_output=True, check=True).stdout.strip())}
    py = str(ROOT.parent / "etesia-quant/.venv/bin/python")  # not resolve(): the venv link must stay
    return {"captured_at": captured_at, "endpoint": re.match(r"https?://[^/]+", inputs["rpc"])[0],
            "ledgers": {"first": min(ledgers), "last": max(ledgers)},
            "revisions": {"vault": git(str(ROOT)), "quant": git(str(ROOT.parent / "etesia-quant")),
                          "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()},
            "tools": {"stellar_cli": version("stellar", "--version"), "node": version("node", "--version"),
                      "stellar_sdk_js": json.loads((ROOT / "tools/source-probe/node_modules/@stellar/stellar-sdk/package.json").read_text())["version"],
                      "python": version(py, "--version"), "stellar_sdk_py": version(py, "-c", "import stellar_sdk;print(stellar_sdk.__version__)")}}


def accounts():
    for name in (ADMIN, TREASURY, REALIGN):
        if (Path(os.environ["STELLAR_CONFIG_HOME"]) / "identity" / f"{name}.toml").exists():
            print("exists", name)
        else:
            command(["keys", "generate", name, "--network", "testnet", "--fund"])
        if not fetch("account", "--account", name):
            command(["keys", "fund", name, "--network", "testnet"])
        state.setdefault("accounts", {})[name] = command(["keys", "address", name])[0]
        save()
        print("funded", name, state["accounts"][name], flush=True)
    line = inputs["assets"]["USDC"]["classic"]
    if "realign_trustline" not in state and fetch("trustline", "--account", REALIGN, "--asset", line):
        sys.exit(f"the {REALIGN} trustline exists but its tx hash is not in {STATE}; recover it from {log}")
    step("realign_trustline", lambda: {"line": line, "hashes": command(
        ["tx", "new", "change-trust", "--source-account", REALIGN, "--line", line, "--network", "testnet"])[1]})
    print("trustline", REALIGN, line, state["realign_trustline"]["hashes"])


def v3():
    for name in (ADMIN, TREASURY, REALIGN):
        assert name in state.get("accounts", {}), "run `accounts` first"
    assert "realign_trustline" in state, "run `accounts` first: the realign trustline hash is not recorded"
    if EVIDENCE.exists():
        assert "v3_copy" not in json.loads(EVIDENCE.read_text()), "v3_copy is already recorded; each key is written once"
    expected = next(line.split()[0] for line in (ROOT / "fixtures/wasm/SHA256SUMS").read_text().splitlines() if line.endswith(WASM.name))
    digest = hashlib.sha256(WASM.read_bytes()).hexdigest()
    assert digest == expected, f"{WASM.name} is {digest}, SHA256SUMS says {expected}"
    captured_at = datetime.now(UTC).isoformat()
    constructor = {"admin": state["accounts"][ADMIN], "treasury": state["accounts"][TREASURY], "protocol_bps": 10, "min_partner_bps": 25,
                   "soroswap_router": inputs["soroswap"]["router"], "aqua_router": inputs["aqua_router"]}
    contract = command(["contract", "id", "wasm", "--salt", SALT, "--source-account", ADMIN, "--network", "testnet"])[0]

    def run_deploy():
        deployed, hashes = deploy(WASM, ADMIN, "testnet", ALIAS, **constructor)
        assert deployed == contract, f"deployed {deployed}, expected {contract} from the salt"
        return {"constructor": constructor, "hashes": hashes}
    step("deploy", run_deploy)
    state["deploy"].update(contract=contract, salt=SALT, constructor=constructor)
    save()
    print("deployed", contract, flush=True)

    def run_extend():
        stdout, hashes = command(["contract", "extend", "--id", contract, "--ledgers-to-extend", str(LEDGERS_TO_EXTEND),
                                  "--source-account", ADMIN, "--network", "testnet"])
        return {"ledgers_to_extend": LEDGERS_TO_EXTEND, "cli_output": stdout, "hashes": hashes}
    print("extended", contract, step("extend", run_extend)["cli_output"], flush=True)

    def run_extend_code():  # the code entry too, in case the instance extension does not carry it
        stdout, hashes = command(["contract", "extend", "--wasm-hash", digest, "--ledgers-to-extend", str(LEDGERS_TO_EXTEND),
                                  "--source-account", ADMIN, "--network", "testnet"])
        return {"wasm_hash": digest, "ledgers_to_extend": LEDGERS_TO_EXTEND, "cli_output": stdout, "hashes": hashes}
    print("extended code", digest, step("extend_code", run_extend_code)["cli_output"], flush=True)
    text = INPUT.read_text()
    if inputs["v3_copy"] != contract:
        assert '"<id after deploy>"' in text, f"testnet.json v3_copy is {inputs['v3_copy']}, not the placeholder"
        INPUT.write_text(text.replace('"<id after deploy>"', json.dumps(contract)))
    evidence = json.loads(EVIDENCE.read_text()) if EVIDENCE.exists() else {"label": LABEL, "network_epoch": EPOCH}
    steps = ("deploy", "extend", "extend_code", "realign_trustline")
    evidence["v3_copy"] = {**meta(captured_at, [t["ledger"] for k in steps for t in state[k]["transactions"]]), "label": "Etesia-deployed copy of Soroswap's mainnet v3 WASM", "contract": contract,
                           "alias": ALIAS, "wasm": {"file": str(WASM.relative_to(ROOT)), "sha256": digest},
                           "constructor": state["deploy"]["constructor"], "accounts": state["accounts"],
                           "transactions": {k: state[k] for k in steps}}
    EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
    EVIDENCE.write_text(json.dumps(evidence, indent=2) + "\n")
    print("recorded v3_copy", contract)


# Both the RPC this script reads and the network the CLI signs for must be testnet.
cli_network = json.loads(command(["network", "info", "--network", "testnet", "--output", "json"])[0])
if PASSPHRASE != rpc("getNetwork")["passphrase"] or PASSPHRASE != inputs["passphrase"] or PASSPHRASE != cli_network["passphrase"]:
    sys.exit("Refuse: testnet only")
{"accounts": accounts, "v3": v3}[sys.argv[1] if len(sys.argv) == 2 else sys.exit("usage: testnet-deploy.py <accounts|v3>")]()
