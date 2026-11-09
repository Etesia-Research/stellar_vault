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
os.environ.setdefault("STELLAR_CONFIG_HOME", str(ROOT / ".stellar"))
RPC = "http://127.0.0.1:8003"
PASSPHRASE = "Standalone Network ; September 2026"
OUT = ROOT / "artifacts/local"
OUT.mkdir(parents=True, exist_ok=True)
log = OUT / "commands.jsonl"
manifest = {"network": "local-only", "passphrase": PASSPHRASE, "rpc": RPC,
            "protocol": 30, "assets": [], "transactions_log": "commands.jsonl", "transactions": [],
            "funding": "public standalone genesis account; no portfolio XLM reimbursement"}


def command(args):
    result = subprocess.run(["stellar", *args], text=True, capture_output=True)
    with log.open("a") as stream:
        stream.write(json.dumps({"args": args, "returncode": result.returncode,
                                 "stdout": result.stdout, "stderr": result.stderr}) + "\n")
    if result.returncode == 0:
        hashes = re.findall(r"(?:Signing transaction|Transaction hash|Transaction Hash):\s*([0-9a-f]{64})", result.stderr)
        if hashes:
            manifest["transactions"].append({"method": args[args.index("--") + 1] if "invoke" in args else args[1], "hash": hashes[-1]})
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


def rpc(method, **params):
    request = urllib.request.Request(RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(), headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(request, timeout=10))["result"]


def save():
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


network = rpc("getNetwork")
assert network["passphrase"] == PASSPHRASE, "Refuse a non-local network"
root = command(["keys", "address", "local-root"])
command(["keys", "generate", "local-fees", "--overwrite"])
recipient = command(["keys", "address", "local-fees"])
base = ROOT / "target/wasm32v1-none/release"
fixture = base / "etesia_local_fixture.wasm"
for symbol, kind in [("USDC", "Settlement"), ("XLM", "Xlm"), ("AQUA", "Risk"), ("ETH", "Risk"), ("BTC", "Risk"), ("USTRY", "Reserve")]:
    address = deploy(fixture, admin=root)
    manifest["assets"].append({"symbol": "LOCAL-" + symbol, "address": address, "kind": kind, "decimals": 7, "issuer": "local fixture, not an issuer-backed asset"})
    invoke(address, "mint", to=root, amount=1_000_000_000_000)
    print("funded", symbol, address, flush=True)
    save()
assets = {a["symbol"].removeprefix("LOCAL-"): a["address"] for a in manifest["assets"]}
oracle = deploy(fixture, admin=root)


def price(asset):
    return 10**12 if asset == assets["USDC"] else int(invoke(oracle, "mark", asset=asset)["price"])


# Mainnet v3, router, factory and pair code (fixtures/wasm/SHA256SUMS).
wasm = ROOT / "fixtures/wasm"
pair_hash = command(["contract", "upload", "--wasm", str(wasm / "soroswap_pair.wasm"), "--source", "local-root", "--network", "local"]).splitlines()[-1]
factory = deploy(wasm / "soroswap_factory.wasm")
invoke(factory, "initialize", setter=root, pair_wasm_hash=pair_hash)
soroswap = deploy(wasm / "soroswap_router.wasm")
invoke(soroswap, "initialize", factory=factory)
command(["keys", "generate", "local-treasury", "--overwrite"])
treasury = command(["keys", "address", "local-treasury"])
PROTOCOL_BPS = 10
# The Aqua router is a placeholder: the vault rejects every non-Soroswap hop.
router = deploy(wasm / "soroswap_aggregator_v3.wasm", admin=root, treasury=treasury, protocol_bps=PROTOCOL_BPS,
                min_partner_bps=PROTOCOL_BPS, soroswap_router=soroswap, aqua_router=treasury)
# One USDC/XLM pair seeded at the fixture mark.
depth = 100_000_000_000
invoke(soroswap, "add_liquidity", token_a=assets["USDC"], token_b=assets["XLM"], amount_a_desired=depth,
       amount_b_desired=depth * 10**12 // price(assets["XLM"]), amount_a_min=0, amount_b_min=0, to=root, deadline=int(time.time()) + 1800)
pair = invoke(factory, "get_pair", token_a=assets["USDC"], token_b=assets["XLM"])
config = {"assets": sorted([{k: a[k] for k in ("address", "decimals", "kind")} for a in manifest["assets"]], key=lambda a: base64.b32decode(a["address"])[1:33]),
          "usdc": assets["USDC"], "oracle": oracle, "router": router, "route_contracts": [pair], "pool": None, "pool_assets": [],
          "admin": root, "executor": root, "guardian": root, "fees": {"management_bps": 100, "performance_bps": 1000, "recipient": recipient},
          "borrowing": False, "max_price_age": 300, "max_slippage_bps": 100,
          "max_leg_bps": 2000, "max_turnover_bps": 10000, "max_loss_bps": 200, "cooldown": 1, "max_debt_bps": 0, "min_health_bps": 12500}
vault = deploy(base / "etesia_vault.wasm", config=config, seed_from=root)
manifest.update(vault=vault, config=config, router=router, oracle=oracle, soroswap={"router": soroswap, "factory": factory, "pair": pair, "treasury": treasury, "protocol_bps": PROTOCOL_BPS})
manifest["wasm_sha256"] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [*base.glob("*.wasm"), *wasm.glob("*.wasm")]}
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


def ceil_div(a, b):
    return -(-a // b)


def floor(input_, output, amount):
    # The smallest minimum that passes the vault's floor (execution.rs) at the fixture mark; all assets have 7 decimals.
    value = ceil_div(amount * price(input_), 10**12)
    return ceil_div(ceil_div(value * (10_000 - config["max_slippage_bps"]), 10_000) * 10**12, price(output))


def swap(input_, output, amount):
    # v3 takes its protocol fee from `amount`; the routes carry the rest.
    return {"Swap": {"token_in": input_, "token_out": output, "amount": str(amount), "minimum": str(floor(input_, output, amount)),
                     "routes": [{"amount_in": str(amount - amount * PROTOCOL_BPS // 10_000), "hops": [{"Soroswap": {"pair": pair, "token_in": input_, "token_out": output}}]}]}}


def swap_output(tx_hash):
    # amount_out of the one v3 `swap` event in the decoded meta.
    meta = rpc("getTransaction", hash=tx_hash, xdrFormat="json")
    events = [e["body"]["v0"] for op in meta["events"]["contractEventsJson"] for e in op
              if e["contract_id"] == router and e["body"]["v0"]["topics"][0] == {"symbol": "swap"}]
    assert len(events) == 1, events
    return int({f["key"]["symbol"]: f["val"] for f in events[0]["data"]["map"]}["amount_out"]["i128"])


plan = {"target": hash_, "epoch": 1, "nonce": 0, "version": 1, "deadline": target["expiry"], "actions": [swap(assets["USDC"], assets["XLM"], amount)]}
invoke(vault, "execute", plan=plan)
output = swap_output(manifest["transactions"][-1]["hash"])
bought = int(invoke(assets["XLM"], "balance", id=vault))
assert bought == output >= int(plan["actions"][0]["Swap"]["minimum"]), (bought, output)
print("guarded swap confirmed", bought, flush=True)
invoke(vault, "collect_fees")
invoke(vault, "unwind", actions=[swap(assets["XLM"], assets["USDC"], bought)], nonce=1, deadline=target["expiry"])
received = int(invoke(vault, "redeem", shares=shares, receiver=root, owner=root, operator=root))
assert int(invoke(vault, "balance", id=root)) == 0
manifest.update(result="passed", deposited_assets=1_000_000_000, minted_shares=shares, redeemed_assets=received, final_state=invoke(vault, "state"), target=target, plan=plan, swap_output=output)
save()
print("local RPC round trip passed", received, flush=True)
