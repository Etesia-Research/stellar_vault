#!/usr/bin/env python3
"""Testnet only: the D4-02 accounts, the realign trustline and the Etesia-deployed copy of Soroswap's mainnet v3 WASM;
the D4-03 keys, trustlines, tRESERVE SAC and pair, pricing provider, vaults, manifest, realigns and records.
Run from etesia-vault/: python3 scripts/testnet-deploy.py <mode>; modes in USAGE, steps in doc/d4-03-implementation-plan.md.

No secret is read: `keys secret` is never run, and command() logs stdout and stderr to the gitignored
artifacts/local/commands.jsonl. Steps already done (identity, funding, trustline, deploy, extensions) are skipped,
and a step's transaction hashes are saved even when the CLI fails after signing, so a rerun checks those
transactions instead of sending new ones. The fixed salt also makes a second deploy of the copy impossible.
"""
import base64
from datetime import UTC, datetime
from decimal import Decimal, getcontext
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import subprocess
import sys
import time
import urllib.error
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
# D4-03
EXECUTOR, GUARDIAN, GUARD_EXECUTOR, GUARD_GUARDIAN = "etesia-executor", "etesia-guardian", "etesia-guard-executor", "etesia-guard-guardian"
MANIFEST = ROOT / f"deploy/testnet/{EPOCH}.json"
REALIGNS = ROOT / f"docs/evidence/d4-realign-{EPOCH}.jsonl"
QUOTE = OUT / "d4-03-quote.json"
KEEPER = OUT / "d4-03-keeper/keeper"
FIXTURES = ROOT / "fixtures/d5-04-testnet"
RELEASE = ROOT / "target/wasm32v1-none/release"
LOCAL_WASM = json.loads((ROOT / "docs/evidence/d4-local-rpc.json").read_text())["wasm_sha256"]
SEED = 10_000_000_000  # 1,000 USDC: the vault constructor's SEED_ASSETS (contracts/vault/src/types.rs)
PAIR_DEPTH = 10_000_000_000  # 1,000 USDC + 1,000 tRESERVE (D4-03-Q6, A71)
SOROSWAP_API = "https://api.soroswap.finance"
USAGE = "usage: testnet-deploy.py <accounts|v3|assets [--funding <hash>...]|quote|vaults|realign <xlm|treserve>|record>"
inputs = json.loads(INPUT.read_text())
USDC, XLM = inputs["assets"]["USDC"]["sac"], inputs["assets"]["XLM"]["sac"]
ROUTER, FACTORY, REFLECTOR = inputs["soroswap"]["router"], inputs["soroswap"]["factory"], inputs["reflector"]["id"]
POOL = inputs["blend"]["pools"]["TestnetV2"]  # A65 T2-Q4
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


def arguments(kwargs):
    args = []
    for key, value in kwargs.items():
        args += ["--" + key.replace("_", "-"), json.dumps(value, separators=(",", ":")) if isinstance(value, (dict, list)) else str(value)]
    return args


def deploy(source, alias, salt, *, wasm=None, wasm_hash=None, **kwargs):
    code = ["--wasm", str(wasm)] if wasm else ["--wasm-hash", wasm_hash]
    stdout, hashes = command(["contract", "deploy", *code, "--optimize=false", "--salt", salt, "--source-account", source,
                              "--network", "testnet", "--alias", alias, "--", *arguments(kwargs)])
    return stdout.splitlines()[-1], hashes


def invoke(contract, method, source=ADMIN, send=True, **kwargs):
    stdout, hashes = command(["contract", "invoke", "--id", contract, "--source-account", source, "--network", "testnet",
                              *([] if send else ["--send=no"]), "--", method, *arguments(kwargs)])
    return json.loads(stdout) if stdout else None, hashes


def view(contract, method, **kwargs):
    return invoke(contract, method, send=False, **kwargs)[0]


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


def identities(names, key):
    for name in names:
        if (Path(os.environ["STELLAR_CONFIG_HOME"]) / "identity" / f"{name}.toml").exists():
            print("exists", name)
        else:
            command(["keys", "generate", name, "--network", "testnet", "--fund"])
        if not fetch("account", "--account", name):
            command(["keys", "fund", name, "--network", "testnet"])
        state.setdefault(key, {})[name] = command(["keys", "address", name])[0]
        save()
        print("funded", name, state[key][name], flush=True)


def trustline(key, name):
    """The Blend USDC trustline of `name`, sent once."""
    line = inputs["assets"]["USDC"]["classic"]
    if key not in state and fetch("trustline", "--account", name, "--asset", line):
        sys.exit(f"the {name} trustline exists but its tx hash is not in {STATE}; recover it from {log}")
    step(key, lambda: {"line": line, "hashes": command(
        ["tx", "new", "change-trust", "--source-account", name, "--line", line, "--network", "testnet"])[1]})
    print("trustline", name, line, state[key]["hashes"])


def accounts():
    identities((ADMIN, TREASURY, REALIGN), "accounts")
    trustline("realign_trustline", REALIGN)


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
        deployed, hashes = deploy(ADMIN, ALIAS, SALT, wasm=WASM, **constructor)
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


# D4-03 (doc/d4-03-implementation-plan.md, A71).
def address(name):
    return {**state.get("accounts", {}), **state.get("identities", {})}[name]


def latest():
    """(sequence, close time in unix seconds) of the latest ledger."""
    r = rpc("getLatestLedger")
    return r["sequence"], int(r["closeTime"])


def evidence_file():
    return json.loads(EVIDENCE.read_text())


def write_evidence(evidence):
    """Append the keys not yet in the file; the bytes of every recorded key stay as they are."""
    text = EVIDENCE.read_text().rstrip()
    recorded = json.loads(text)
    assert text.endswith("}") and all(evidence[k] == v for k, v in recorded.items()), "recorded keys are written once"
    added = "".join(f',\n  {json.dumps(k)}: {json.dumps(v, indent=2).replace(chr(10), chr(10) + "  ")}'
                    for k, v in evidence.items() if k not in recorded)
    EVIDENCE.write_text(text[:-1].rstrip() + added + "\n}\n")
    assert json.loads(EVIDENCE.read_text()) == evidence


def executable(contract):
    """The code hash of a contract instance, or "stellar-asset" for a SAC; None when the instance is missing."""
    entries = fetch("contract-data", "--contract", contract, "--instance")
    if not entries:
        return None
    code = entries[0]["val"]["contract_data"]["val"]["contract_instance"]["executable"]
    return "stellar-asset" if code == "stellar_asset" else code["wasm"]


def payment(tx_hash):
    """One user-sent faucet payment of Blend USDC to an Etesia account, read from RPC."""
    r = rpc("getTransaction", hash=tx_hash, xdrFormat="json")
    assert r["status"] == "SUCCESS", f"{tx_hash}: {r['status']}"
    tx = r["envelopeJson"]["tx"]["tx"]
    code, issuer = inputs["assets"]["USDC"]["classic"].split(":")
    ours = {v: k for k, v in state["accounts"].items()}
    rows = []
    for op in tx["operations"]:
        p = op["body"]["payment"]
        assert p["asset"] == {"credit_alphanum4": {"asset_code": code, "issuer": issuer}} and p["destination"] in ours, (tx_hash, p)
        rows.append({"from": op["source_account"] or tx["source_account"], "to": p["destination"], "to_name": ours[p["destination"]],
                     "amount": p["amount"]})
    return {"label": "Blend testnet faucet funding: claimed by the user from a browser wallet and paid to an Etesia account (A71, D4-03-Q5)",
            "hash": tx_hash, "ledger": r["ledger"], "created_at": datetime.fromtimestamp(int(r["createdAt"]), UTC).isoformat(),
            "payments": rows}


def assets(funding):
    for name in (ADMIN, TREASURY, REALIGN):
        assert name in state.get("accounts", {}), "run `accounts` first"
    identities((EXECUTOR, GUARDIAN, GUARD_EXECUTOR, GUARD_GUARDIAN), "identities")
    trustline("admin_trustline", ADMIN)
    trustline("treasury_trustline", TREASURY)
    need = {ADMIN: SEED * sum(k not in state for k in ("flagship", "guard")),
            REALIGN: PAIR_DEPTH * ("treserve_pair" not in state)}
    held = {name: int(view(USDC, "balance", id=address(name))) for name in need}
    if any(held[name] < need[name] for name in need):
        sys.exit(f"stop: Blend USDC held {held}, needed {need} (base units); fund through the faucet (D4-03-Q5), then rerun")
    if "funding" not in state:
        if not funding:
            sys.exit("pass the faucet payment hashes: assets --funding <hash>...")
        state["funding"] = [payment(h) for h in funding]
        save()
    issuer = address(REALIGN)
    asset = f"tRESERVE:{issuer}"
    sac = command(["contract", "id", "asset", "--asset", asset, "--network", "testnet"])[0]
    if "treserve_sac" not in state and (found := executable(sac)):
        assert found == "stellar-asset", (sac, found)
        state["treserve_sac"] = {"asset": asset, "contract": sac, "existing": True, "hashes": [], "transactions": []}
        save()

    def run_sac():
        stdout, hashes = command(["contract", "asset", "deploy", "--asset", asset, "--source-account", REALIGN,
                                  "--network", "testnet", "--alias", "etesia-treserve"])
        assert stdout.splitlines()[-1] == sac, (stdout, sac)
        return {"asset": asset, "contract": sac, "hashes": hashes}
    step("treserve_sac", run_sac)
    state["treserve_sac"].update(asset=asset, contract=sac)  # also when the CLI failed after signing
    save()
    assert executable(sac) == "stellar-asset", sac
    print("tRESERVE SAC", sac, flush=True)

    def run_pair():
        deadline = latest()[1] + 300
        returned, hashes = invoke(ROUTER, "add_liquidity", source=REALIGN, token_a=USDC, token_b=sac,
                                  amount_a_desired=PAIR_DEPTH, amount_b_desired=PAIR_DEPTH, amount_a_min=PAIR_DEPTH,
                                  amount_b_min=PAIR_DEPTH, to=issuer, deadline=deadline)
        return {"returned": returned, "deadline": deadline, "hashes": hashes}
    step("treserve_pair", run_pair)
    pair = view(FACTORY, "get_pair", token_a=USDC, token_b=sac)
    state["treserve_pair"].update(pair=pair, sac=sac)
    save()
    reads = {"pair": pair, "token_0": view(pair, "token_0"), "token_1": view(pair, "token_1"),
             "reserves": view(pair, "get_reserves"), "lp_held_by_issuer": view(pair, "balance", id=issuer),
             "ledger": latest()[0]}
    print("tRESERVE pair", reads, flush=True)
    evidence = evidence_file()
    if "assets" in evidence:
        print("assets already recorded")
        return
    keys = ("admin_trustline", "treasury_trustline", "treserve_sac", "treserve_pair")
    ledgers = [t["ledger"] for k in keys for t in state[k]["transactions"]] + [f["ledger"] for f in state["funding"]] + [reads["ledger"]]
    evidence["assets"] = {**meta(datetime.now(UTC).isoformat(), ledgers),
                          "label": "Etesia-issued tRESERVE (reserve stand-in, marked at par to Reflector USDC) and its Etesia-seeded Soroswap pair; Blend testnet USDC from the faucet",
                          "transactions": {k: state[k] for k in keys}, "funding": state["funding"],
                          "treserve": {"asset": asset, "sac": sac, "issuer_flags": "none (no AUTH_REQUIRED, AUTH_REVOCABLE or AUTH_CLAWBACK)"},
                          "pair": {**reads, "lp_minimum_liquidity_burned": 1000}}
    write_evidence(evidence)
    print("recorded assets")


def quote():
    """Poll POST /quote until the tRESERVE pair routes; writes only QUOTE (never STATE or the evidence file)."""
    key = os.environ["SOROSWAP_API_KEY"]
    sac = state["treserve_sac"]["contract"]
    start = datetime.fromisoformat(state["treserve_pair"]["transactions"][0]["created_at"])

    def post(amount):
        body = {"assetIn": USDC, "assetOut": sac, "amount": str(amount), "tradeType": "EXACT_IN", "protocols": ["soroswap"],
                "parts": 1, "maxHops": 1, "slippageBps": 50}  # routing.quote, P6
        request = urllib.request.Request(SOROSWAP_API + "/quote?network=testnet", data=json.dumps(body).encode(), headers={
            "Content-Type": "application/json", "Authorization": "Bearer " + key, "User-Agent": "etesia-testnet-deploy"})
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return body, response.status, response.read().decode()
        except urllib.error.HTTPError as error:
            return body, error.code, error.read().decode()
        except OSError as error:
            return body, None, str(error)

    def indexed(status, text):
        try:
            distribution = json.loads(text)["rawTrade"]["distribution"]
            return status in (200, 201) and len(distribution) == 1 and distribution[0]["protocol_id"] == "soroswap" \
                and distribution[0]["path"] == [USDC, sac]
        except (ValueError, KeyError, TypeError, IndexError):
            return False

    polls, last = 0, None
    while True:
        polls += 1
        request, status, text = post(100_000_000)  # 10 USDC
        at = datetime.now(UTC)
        if indexed(status, text):
            break
        last = {"status": status, "body": text[:2000], "at": at.isoformat()}
        print("poll", polls, status, text[:200], flush=True)
        if (at - start).total_seconds() > 48 * 3600:
            QUOTE.write_text(json.dumps({"indexed": None, "polls": polls, "last_not_indexed": last, "from": start.isoformat()}, indent=2) + "\n")
            sys.exit("gave up after 48 h: the API has not indexed the tRESERVE pair")
        time.sleep(60)
    # The planner's first leg is 200 USDC; the keeper quotes it net of the copy's fee (routing.leg).
    gross = 2_000_000_000
    net = gross - gross * view(inputs["v3_copy"], "config")["protocol_bps"] // 10_000
    request_200, status_200, text_200 = post(net)
    QUOTE.write_text(json.dumps({
        "label": "Soroswap API /quote index check of the Etesia-seeded tRESERVE pair",
        "url": SOROSWAP_API + "/quote?network=testnet", "from": start.isoformat(), "polls": polls, "poll_seconds": 60,
        "indexed": {"request": request, "status": status, "body": text, "at": at.isoformat()},
        "elapsed_s": (at - start).total_seconds(), "last_not_indexed": last,
        "leg_200_usdc": {"gross": gross, "request": request_200, "status": status_200, "body": text_200,
                         "at": datetime.now(UTC).isoformat()}}, indent=2) + "\n")
    print("indexed after", polls, "polls,", (at - start).total_seconds(), "s; 200-USDC quote status", status_200)


def deployed(key, alias, code_hash, **kwargs):
    """Deploy once from etesia-admin with salt sha256(alias); `config` (if any) is kept in STATE. A `--wasm` deploy of
    code not yet on chain signs the upload first, so the deploy is the step's last transaction."""
    salt = hashlib.sha256(alias.encode()).hexdigest()
    contract = command(["contract", "id", "wasm", "--salt", salt, "--source-account", ADMIN, "--network", "testnet"])[0]

    def run():
        got, hashes = deploy(ADMIN, alias, salt, **kwargs)
        assert got == contract, f"deployed {got}, expected {contract} from the salt"
        return {"hashes": hashes}
    record = step(key, run)
    if executable(contract) != code_hash:  # e.g. only the upload landed before the CLI failed
        del state[key]
        save()
        sys.exit(f"{key}: no {code_hash} instance at {contract}; rerun to deploy it")
    record.update(alias=alias, contract=contract, salt=salt, arguments=record.get("arguments") or
                  {k: v for k, v in kwargs.items() if k not in ("wasm", "wasm_hash")})
    save()
    print("deployed", key, contract, flush=True)
    return record


def vaults():
    assert "transactions" in state.get("treserve_pair", {}) and "pair" in state["treserve_pair"], "run `assets` first: the tRESERVE pair is not recorded"
    evidence = evidence_file()
    if "vaults" in evidence:
        sys.exit("vaults is already recorded; each key is written once")
    captured_at = datetime.now(UTC).isoformat()
    sac, treserve_pair, admin = state["treserve_sac"]["contract"], state["treserve_pair"]["pair"], address(ADMIN)
    d4 = evidence["code"]["contracts"]
    xlm_pair = evidence["xlm_pair"]["pair"]
    copy = inputs["v3_copy"]
    # (1) The Etesia WASMs are the D4-01 local-run builds.
    command(["contract", "build", "--locked", "--optimize"])
    build = {n: {"sha256": hashlib.sha256((RELEASE / n).read_bytes()).hexdigest(), "expected": LOCAL_WASM[n]}
             for n in ("etesia_vault.wasm", "etesia_pricing.wasm")}
    assert all(b["sha256"] == b["expected"] for b in build.values()), build
    # (2) One provider for both vaults (the observed Reflector v7 feed, A68 max_age).
    feed = evidence["reflector"]["contract"]
    assert (feed["base"], feed["decimals"], feed["resolution"]) == (["Other", "USD"], 14, 300), feed

    def reflector(symbol):
        return {"contract": REFLECTOR, "asset": {"Other": symbol}, "base": {"Other": "USD"}, "decimals": 14, "resolution": 300, "max_age": 3600}
    provider_config = {"usdc": USDC, "sources": [{"asset": XLM, "numerator": reflector("XLM"), "denominator": [reflector("USDC")]},
                                                 {"asset": sac, "numerator": reflector("USDC"), "denominator": [reflector("USDC")]}]}
    provider = deployed("provider", "etesia-pricing", LOCAL_WASM["etesia_pricing.wasm"], wasm=RELEASE / "etesia_pricing.wasm", c=provider_config)["contract"]
    deployed_config = view(provider, "config")
    marks = {"XLM": view(provider, "mark", asset=XLM), "tRESERVE": view(provider, "mark", asset=sac), "ledger": latest()[0]}
    assert int(marks["XLM"]["price"]) > 0 and int(marks["tRESERVE"]["price"]) == 10**12, marks
    # (3) What the constructor does not check; a wrong id would lock the vault's seed.
    reserve = view(POOL, "get_reserve", asset=USDC)
    aggregator_config = view(copy, "config")
    checks = [("provider config() equals its input", provider_config, deployed_config),
              ("provider usdc = vault usdc", USDC, deployed_config["usdc"]),
              ("route_contracts[0] = factory get_pair(USDC, XLM)", xlm_pair, view(FACTORY, "get_pair", token_a=USDC, token_b=XLM)),
              ("route_contracts[1] = factory get_pair(USDC, tRESERVE)", treserve_pair, view(FACTORY, "get_pair", token_a=USDC, token_b=sac)),
              ("pool get_reserve(USDC) decimals", 7, reserve["config"]["decimals"]),
              ("pool get_reserve(USDC) scalar", "10000000", reserve["scalar"]),
              ("pool USDC bToken emission entry", None, view(POOL, "get_reserve_emissions", reserve_token_index=reserve["config"]["index"] * 2 + 1)),
              ("router (v3 copy) not paused", False, aggregator_config["paused"])]
    prechecks = [{"check": c, "expected": e, "observed": o, "ok": e == o} for c, e, o in checks]
    prechecks.append({"ledger": latest()[0]})
    if not all(r.get("ok", True) for r in prechecks):
        sys.exit(f"pre-check failed; nothing deployed after the provider: {json.dumps(prechecks, indent=2)}")
    # (4) Flagship and guard (two of the four seeds, D4-03-Q2).
    order = sorted([{"address": USDC, "decimals": 7, "kind": "Settlement"}, {"address": XLM, "decimals": 7, "kind": "Xlm"},
                    {"address": sac, "decimals": 7, "kind": "Reserve"}], key=lambda a: base64.b32decode(a["address"])[1:33])

    def config(executor, guardian, max_price_age, max_leg_bps, max_turnover_bps, max_loss_bps, cooldown):
        return {"assets": order, "usdc": USDC, "oracle": provider, "router": copy, "route_contracts": [xlm_pair, treserve_pair],
                "pool": POOL, "pool_assets": [USDC], "admin": admin, "executor": address(executor), "guardian": address(guardian),
                "fees": {"management_bps": 100, "performance_bps": 1000, "recipient": admin}, "borrowing": False,
                "max_price_age": max_price_age, "max_slippage_bps": 100, "max_leg_bps": max_leg_bps,
                "max_turnover_bps": max_turnover_bps, "max_loss_bps": max_loss_bps, "cooldown": cooldown, "max_debt_bps": 0,
                "min_health_bps": 12500}
    configs = {"flagship": config(EXECUTOR, GUARDIAN, 3600, 2000, 10_000, 200, 12),
               "guard": config(GUARD_EXECUTOR, GUARD_GUARDIAN, 300, 500, 300, 50, 720)}
    vault_hash = LOCAL_WASM["etesia_vault.wasm"]
    ids = {"flagship": deployed("flagship", "etesia-flagship", vault_hash, wasm=RELEASE / "etesia_vault.wasm", config=configs["flagship"], seed_from=admin)["contract"],
           "guard": deployed("guard", "etesia-guard", vault_hash, wasm_hash=vault_hash, config=configs["guard"], seed_from=admin)["contract"]}
    for name in ids:
        assert state[name]["arguments"]["config"] == configs[name], f"{name} was deployed with another config; see {STATE}"
    # (5) Each vault's config() equals its constructor input.
    equal = {"provider": deployed_config == provider_config}
    for name, contract in ids.items():
        read = view(contract, "config")
        equal[name] = read == configs[name]
        if not equal[name]:
            sys.exit(f"{name} config() differs from its input:\n{json.dumps(read, indent=2)}")
    # (6) Code hashes against the chain.
    expected = {ids["flagship"]: vault_hash, ids["guard"]: vault_hash, provider: LOCAL_WASM["etesia_pricing.wasm"],
                copy: d4["v3_copy"]["wasm_hash"], ROUTER: d4["router"]["wasm_hash"], FACTORY: d4["factory"]["wasm_hash"],
                xlm_pair: d4["xlm_pair"]["wasm_hash"], treserve_pair: d4["xlm_pair"]["wasm_hash"], POOL: d4["pool:TestnetV2"]["wasm_hash"],
                REFLECTOR: d4["reflector"]["wasm_hash"], USDC: "stellar-asset", XLM: "stellar-asset", sac: "stellar-asset"}
    code = {contract: executable(contract) for contract in expected}
    code_rows = [{"contract": c, "expected": e, "observed": code[c], "ok": code[c] == e} for c, e in expected.items()]
    if not all(r["ok"] for r in code_rows):
        sys.exit(f"code hash mismatch: {json.dumps(code_rows, indent=2)}")
    code_ledger = latest()[0]
    # (7) The manifest, written once; quant pins its bytes.
    symbols = {USDC: ("USDC", inputs["assets"]["USDC"]["classic"], "Blend testnet USDC"), XLM: ("XLM", "native", "native XLM SAC"),
               sac: ("tRESERVE", state["treserve_sac"]["asset"], "Etesia-issued reserve stand-in, marked at par to Reflector USDC")}
    manifest = {
        "label": "Testnet rehearsal deployment; Etesia-made and labelled markets; Etesia-deployed copy of Soroswap's mainnet v3 WASM; no mainnet claim",
        "network_epoch": EPOCH, "passphrase": PASSPHRASE, "aggregator": copy, "aggregator_config": aggregator_config, "factory": FACTORY,
        "pairs": [{"address": xlm_pair, "label": "third-party Soroswap pair, realigned by Etesia"},
                  {"address": treserve_pair, "label": "Etesia-seeded pair"}],
        "assets": [{"address": a["address"], "symbol": symbols[a["address"]][0], "kind": a["kind"], "decimals": a["decimals"],
                    "classic": symbols[a["address"]][1], "stand_in": symbols[a["address"]][2]} for a in order],
        "provider": {"id": provider, "config": deployed_config},
        "vaults": {name: {"id": ids[name], "ledger": state[name]["transactions"][-1]["ledger"], **configs[name]} for name in ids},
        "code": code}
    if not MANIFEST.exists():
        MANIFEST.parent.mkdir(parents=True, exist_ok=True)
        MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n")
    digest = hashlib.sha256(MANIFEST.read_bytes()).hexdigest()
    keys = ("provider", "flagship", "guard")
    ledgers = [t["ledger"] for k in keys for t in state[k]["transactions"]] + [marks["ledger"], prechecks[-1]["ledger"], code_ledger]
    evidence["vaults"] = {**meta(captured_at, ledgers), "label": "Etesia rehearsal pricing provider, flagship and guard vaults (pre-reset seeds)",
                          "build": {"command": "stellar contract build --locked --optimize", "wasm": build},
                          "transactions": {k: {f: v for f, v in state[k].items() if f != "arguments"} for k in keys},
                          "seed_from": {"account": admin, "assets": SEED}, "marks": marks, "prechecks": prechecks,
                          "config_equals_manifest": equal, "code": code_rows,
                          "manifest": {"path": str(MANIFEST.relative_to(ROOT)), "sha256": digest}}
    write_evidence(evidence)
    print("recorded vaults; manifest", MANIFEST.relative_to(ROOT), digest)


def realign(which):
    """One Soroswap router trade that moves the pair's mid onto its target (Reflector XLM/USDC, or 1 for tRESERVE)."""
    getcontext().prec = 60
    other = XLM if which == "xlm" else state["treserve_sac"]["contract"]
    pair = evidence_file()["xlm_pair"]["pair"] if which == "xlm" else state["treserve_pair"]["pair"]
    me = address(REALIGN)

    def reserves():
        r = [int(x) for x in view(pair, "get_reserves")]
        return (r[0], r[1]) if view(pair, "token_0") == USDC else (r[1], r[0])  # (USDC, other)

    def side(u, x, target):
        mid = Decimal(u) / Decimal(x)  # USDC per unit; both tokens have 7 decimals
        return {"usdc_reserve": u, "other_reserve": x, "mid": str(mid), "deviation_bps": str((mid / target - 1) * 10_000)}

    if which == "xlm":
        rounds = {s: view(REFLECTOR, "lastprice", asset={"Other": s}) for s in ("XLM", "USDC")}
        target = Decimal(rounds["XLM"]["price"]) / Decimal(rounds["USDC"]["price"])
    else:
        rounds, target = None, Decimal(1)
    u, x = reserves()
    ledger, now = latest()
    # Sell the overpriced side: a, b are the sold and bought reserves and p the target b/a.
    sold, bought, a, b, p = (other, USDC, x, u, target) if Decimal(u) / Decimal(x) > target else (USDC, other, u, x, 1 / target)
    g = Decimal("0.997")
    c = Decimal(a) * a - Decimal(a) * b / p
    amount = int((-(1 + g) * a + ((1 + g) ** 2 * a * a - 4 * g * c).sqrt()) / (2 * g))
    out = amount * 997 * b // (a * 1000 + amount * 997)
    record = {"label": "Etesia realign trade", "network_epoch": EPOCH, "pair": which, "pair_address": pair, "venue": ROUTER,
              "reflector": rounds, "target_usdc_per_unit": str(target), "before": {**side(u, x, target), "ledger": ledger},
              "sold": sold, "bought": bought, "amount_in": amount, "expected_out": out, "amount_out_min": out * 9_990 // 10_000,
              "deadline": now + 120, "source": me}

    def append(line):
        with REALIGNS.open("a") as stream:
            stream.write(json.dumps({**line, "written_at": datetime.now(UTC).isoformat()}) + "\n")
    if amount <= 0:
        append({**record, "status": "no trade"})
        print("no trade", record["before"])
        return
    print("realign", which, "sell", amount, sold, "for >=", record["amount_out_min"], bought, flush=True)
    try:
        returned, hashes = invoke(ROUTER, "swap_exact_tokens_for_tokens", source=REALIGN, amount_in=amount,
                                  amount_out_min=record["amount_out_min"], path=[sold, bought], to=me, deadline=record["deadline"])
        failure = None
    except CliError as error:
        returned, hashes, failure = None, error.hashes, str(error)[-2000:]
    append({**record, "status": "signed", "hashes": hashes, "cli_error": failure})
    if not hashes:
        sys.exit(f"no transaction signed: {failure}")
    rows = included(hashes)
    if rows is None:
        append({**record, "status": "failed or never landed", "hashes": hashes})
        sys.exit("the realign failed or never landed")
    u, x = reserves()
    append({**record, "status": "included", "returned": returned, "transactions": rows,
            "after": {**side(u, x, target), "ledger": latest()[0]}})
    print("realigned", which, side(u, x, target))


def record():
    """D5-04 fixtures for the flagship deploy and the keeper's included attempts; the rehearsal and quote_index keys."""
    evidence = evidence_file()
    flagship = state["flagship"]["contract"]
    FIXTURES.mkdir(parents=True, exist_ok=True)

    def fixture(n, method, tx_hash):
        path = FIXTURES / f"{n:02d}-{method}.json"
        if not path.exists():
            receipt = rpc("getTransaction", hash=tx_hash)
            assert receipt["status"] == "SUCCESS", f"{tx_hash}: {receipt['status']} (RPC keeps about 7 days)"
            decoded = rpc("getTransaction", hash=tx_hash, xdrFormat="json")
            assert flagship in json.dumps(decoded), f"{tx_hash} does not touch the flagship"
            path.write_text(json.dumps({"method": method, "hash": tx_hash, "receipt": receipt, "decoded": decoded}, separators=(",", ":")) + "\n")
            print("fixture", path.relative_to(ROOT))
        return str(path.relative_to(ROOT))
    deploy_tx = state["flagship"]["transactions"][-1]["hash"]  # after the WASM upload
    fixtures = {deploy_tx: fixture(0, "deploy", deploy_tx)}
    journal = KEEPER / "journal.sqlite3"
    if journal.exists():
        db = sqlite3.connect(journal)
        db.row_factory = sqlite3.Row
        attempts = [dict(r) for r in db.execute("SELECT * FROM attempts WHERE vault = ? AND outcome IS NOT NULL ORDER BY resolved_order", (flagship,))]
        for n, row in enumerate([r for r in attempts if r["outcome"] == "success"], start=1):
            fixtures[row["tx_hash"]] = fixture(n, "publish_target" if row["kind"] == "publish" else "execute", row["tx_hash"])
        executed = any(r["kind"] == "execute" and r["outcome"] == "success" for r in attempts)
        if executed and "rehearsal" not in evidence:
            rows = []
            for row in attempts:
                content = json.loads((KEEPER / "evidence" / Path(row["evidence"]).name).read_text())
                context = json.loads(row["context"])
                item = {k: row[k] for k in ("kind", "tx_hash", "outcome", "classification", "action", "ledger", "nonce", "version",
                                            "target_epoch", "target_hash", "action_hash", "max_time", "send_status")}
                item["keeper_evidence"] = Path(row["evidence"]).name
                if row["kind"] == "publish":
                    item["target"] = context["target"]
                else:
                    item.update(plan=context["plan"], reconciliation=content.get("reconciliation"))
                if row["tx_hash"] in fixtures:
                    item["fixture"] = fixtures[row["tx_hash"]]
                else:
                    item["xdr"] = {k: content[k] for k in ("envelope", "envelopeXdr", "resultXdr", "resultMetaXdr", "errorResultXdr") if k in content}
                rows.append(item)
            cycles = [{**dict(r), "skips": json.loads(r["skips"]), "triggers": json.loads(r["triggers"]) if r["triggers"] else None}
                      for r in db.execute("SELECT * FROM heartbeats WHERE vault = ? ORDER BY cycle", (flagship,))]
            alerts = [dict(r) for r in db.execute("SELECT * FROM alerts WHERE vault = ? ORDER BY id", (flagship,))]
            ledgers = [r["ledger"] for r in attempts if r["ledger"]]
            evidence["rehearsal"] = {**meta(datetime.now(UTC).isoformat(), ledgers),
                                     "label": "Flagship rehearsal: the unchanged keeper, one `once` cycle per run, D4-Q4 `flat` scenario target (D4-03-Q4, A71)",
                                     "vault": flagship, "attempts": rows, "cycles": cycles, "alerts": alerts,
                                     "fixtures": sorted(set(fixtures.values()))}
            print("recorded rehearsal")
    if QUOTE.exists() and "quote_index" not in evidence:
        result = json.loads(QUOTE.read_text())
        if result.get("indexed"):
            evidence["quote_index"] = {**meta(datetime.now(UTC).isoformat(), [state["treserve_pair"]["transactions"][0]["ledger"]]), **result}
            print("recorded quote_index")
    write_evidence(evidence)


# Both the RPC this script reads and the network the CLI signs for must be testnet.
cli_network = json.loads(command(["network", "info", "--network", "testnet", "--output", "json"])[0])
if PASSPHRASE != rpc("getNetwork")["passphrase"] or PASSPHRASE != inputs["passphrase"] or PASSPHRASE != cli_network["passphrase"]:
    sys.exit("Refuse: testnet only")
mode, rest = (sys.argv[1], sys.argv[2:]) if len(sys.argv) > 1 else (None, [])
if mode == "assets" and (not rest or rest[0] == "--funding"):
    assets(rest[1:])
elif mode == "realign" and rest in (["xlm"], ["treserve"]):
    realign(rest[0])
elif mode in ("accounts", "v3", "quote", "vaults", "record") and not rest:
    {"accounts": accounts, "v3": v3, "quote": quote, "vaults": vaults, "record": record}[mode]()
else:
    sys.exit(USAGE)
