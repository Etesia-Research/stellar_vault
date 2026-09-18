#!/usr/bin/env python3
"""Local RPC only. Uses public standalone genesis funds and explicit test assets."""
import base64
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RPC = "http://127.0.0.1:8003"
PASSPHRASE = "Standalone Network ; September 2026"
OUT = ROOT / "artifacts/local"
OUT.mkdir(parents=True, exist_ok=True)
log = OUT / "commands.jsonl"
manifest = {"network": "local-only", "passphrase": PASSPHRASE, "rpc": RPC,
            "protocol": 22, "assets": [], "transactions_log": "commands.jsonl", "transactions": [],
            "funding": "public standalone genesis account; no portfolio XLM reimbursement"}


def command(args):
    result = subprocess.run(["stellar", *args], text=True, capture_output=True)
    with log.open("a") as stream:
        stream.write(json.dumps({"args": args, "returncode": result.returncode,
                                 "stdout": result.stdout, "stderr": result.stderr}) + "\n")
    if result.returncode == 0:
        hashes = re.findall(r"(?:Signing transaction|Transaction hash|Transaction Hash):\s*([0-9a-f]{64})", result.stderr)
        if hashes:
            manifest["transactions"].append({"method": args[args.index("--") + 1] if "invoke" in args else "deploy", "hash": hashes[-1]})
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result.stdout.strip()


def invoke(contract, method, **kwargs):
    args = ["contract", "invoke", "--id", contract, "--source", "local-root", "--network", "local", "--cost", "--", method]
    for key, value in kwargs.items():
        args += ["--" + key.removesuffix("_").replace("_", "-"), json.dumps(value, separators=(",", ":")) if isinstance(value, (dict, list)) else str(value)]
    result = command(args)
    return json.loads(result) if result else None


def deploy(wasm, **kwargs):
    args = ["contract", "deploy", "--wasm", str(wasm), "--source", "local-root", "--network", "local", "--"]
    for key, value in kwargs.items():
        args += ["--" + key.removesuffix("_").replace("_", "-"), json.dumps(value, separators=(",", ":")) if isinstance(value, (dict, list)) else str(value)]
    return command(args).splitlines()[-1]


def save():
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


network = json.loads(urllib.request.urlopen(urllib.request.Request(RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "getNetwork"}).encode(), headers={"Content-Type": "application/json"})).read())["result"]
assert network["passphrase"] == PASSPHRASE, "Refuse a non-local network"
root = command(["keys", "address", "local-root"])
command(["keys", "generate", "local-fees", "--no-fund", "--overwrite"])
recipient = command(["keys", "address", "local-fees"])
base = ROOT / "target/wasm32-unknown-unknown/release"
fixture = base / "etesia_local_fixture.optimized.wasm"
for symbol, kind in [("USDC", "Settlement"), ("XLM", "Xlm"), ("AQUA", "Risk"), ("ETH", "Risk"), ("BTC", "Risk"), ("USTRY", "Reserve")]:
    address = deploy(fixture, admin=root)
    manifest["assets"].append({"symbol": "LOCAL-" + symbol, "address": address, "kind": kind, "decimals": 7, "issuer": "local fixture, not an issuer-backed asset"})
    invoke(address, "mint", to=root, amount=1_000_000_000_000)
    print("funded", symbol, address, flush=True)
    save()
assets = {a["symbol"].removeprefix("LOCAL-"): a["address"] for a in manifest["assets"]}
router = deploy(fixture, admin=root)
for asset in assets.values():
    invoke(asset, "mint", to=router, amount=1_000_000_000_000)
config = {"assets": sorted([{k: a[k] for k in ("address", "decimals", "kind")} for a in manifest["assets"]], key=lambda a: base64.b32decode(a["address"])[1:33]),
          "usdc": assets["USDC"], "oracle": router, "router": router, "route_contracts": [router], "pool": None, "pool_assets": [],
          "admin": root, "executor": root, "guardian": root, "fees": {"management_bps": 100, "performance_bps": 1000, "recipient": recipient},
          "borrowing": False, "max_price_age": 300, "max_divergence_bps": 100, "max_slippage_bps": 100,
          "max_leg_bps": 2000, "max_turnover_bps": 10000, "max_loss_bps": 200, "cooldown": 1, "max_debt_bps": 0, "min_health_bps": 12500}
vault = deploy(base / "etesia_vault.optimized.wasm", config=config, seed_from=root)
manifest.update(vault=vault, config=config, router=router, oracle=router)
manifest["wasm_sha256"] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in base.glob("*.optimized.wasm")}
save()
shares = int(invoke(vault, "deposit", assets=1_000_000_000, receiver=root, from_=root, operator=root))
print("deposited", shares, flush=True)
# CLI spells the Rust argument `from`, not Python's keyword workaround.
target = {"network": hashlib.sha256(PASSPHRASE.encode()).hexdigest(), "vault": vault, "epoch": 1, "expiry": int(time.time()) + 1800,
          "model": "01" * 32, "dataset": "02" * 32, "holdings": invoke(vault, "holdings_hash"),
          "assets": [a["address"] for a in config["assets"]], "weights": [250 if a["kind"] in ("Settlement", "Xlm") else 9500 if a["kind"] == "Reserve" else 0 for a in config["assets"]],
          "yield_bps": 0, "equity": str(invoke(vault, "total_assets")), "supply": str(invoke(vault, "total_supply"))}
hash_ = invoke(vault, "publish_target", target=target)
amount = int(target["equity"]) // 40

def swap(input_, output):
    return {"Swap": {"token_in": input_, "token_out": output, "amount": str(amount), "minimum": str(amount),
                     "distribution": [{"protocol_id": 0, "path": [input_, output], "parts": 1, "bytes": None}],
                     "auth": [{"Transfer": [input_, router, str(amount)]}]}}

plan = {"target": hash_, "epoch": 1, "nonce": 0, "version": 1, "deadline": target["expiry"], "actions": [swap(assets["USDC"], assets["XLM"])]}
invoke(vault, "execute", plan=plan)
assert int(invoke(assets["XLM"], "balance", id=vault)) == amount
print("guarded swap confirmed", flush=True)
invoke(vault, "collect_fees")
invoke(vault, "unwind", actions=[swap(assets["XLM"], assets["USDC"])], nonce=1, deadline=target["expiry"])
received = int(invoke(vault, "redeem", shares=shares, receiver=root, owner=root, operator=root))
assert int(invoke(vault, "balance", id=root)) == 0
manifest.update(result="passed", deposited_assets=1_000_000_000, minted_shares=shares, redeemed_assets=received, final_state=invoke(vault, "state"), target=target, plan=plan)
save()
print("local RPC round trip passed", received, flush=True)
