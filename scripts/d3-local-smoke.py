#!/usr/bin/env python3
"""Local RPC only. Uses public standalone genesis funds and explicit test assets."""
import base64
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RPC = "http://127.0.0.1:8003"
PASSPHRASE = "Standalone Network ; September 2026"
CONVERSION = "--conversion" in sys.argv
OUT = ROOT / ("artifacts/local/d3-conversion" if CONVERSION else "artifacts/local/d3")
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


network = json.loads(urllib.request.urlopen(urllib.request.Request(RPC, data=json.dumps({"jsonrpc":"2.0","id":1,"method":"getNetwork"}).encode(), headers={"Content-Type":"application/json"})).read())["result"]
assert network["passphrase"] == PASSPHRASE, "Refuse a non-local network"
root = command(["keys", "address", "local-root"])
base = ROOT / "target/wasm32-unknown-unknown/release"
fixture = base / "etesia_local_fixture.optimized.wasm"
pair_wasm = ROOT / "artifacts/local/upstream/18051456816b66f12e773a56f77c5794fac1b1fb7ab6e22d4fad5a412770f73e.wasm"
assert hashlib.sha256(pair_wasm.read_bytes()).hexdigest() == pair_wasm.stem
if CONVERSION:
    previous=json.loads((ROOT/"artifacts/local/d3/manifest.json").read_text())
    assert previous["result"]=="passed"
    assert {a["symbol"] for a in previous["assets"]} == {"USDC", "XLM", "AQUA", "ETH", "BTC", "USTRY"}, "Rerun the direct scenario with the current universe"
    manifest["assets"]=previous["assets"]
    assets={a["symbol"]:a["address"] for a in manifest["assets"]}
    sources=previous["provider_config"]["sources"]
    feed=sources[0]["numerator"]["contract"]
    invoke(feed,"update",asset={"Stellar":assets["USDC"]},price=100_000_000_000_000,timestamp=int(time.time()))
    for s in sources:
        s["denominator"]=[{**s["numerator"],"asset":{"Stellar":assets["USDC"]}}]
        invoke(feed,"update",asset={"Stellar":s["asset"]},price=100_000_000_000_000,timestamp=int(time.time()))
else:
    for symbol, kind in [("USDC","Settlement"),("XLM","Xlm"),("AQUA","Risk"),("ETH","Risk"),("BTC","Risk"),("USTRY","Reserve")]:
        address=deploy(fixture,admin=root)
        manifest["assets"].append({"symbol":symbol,"address":address,"decimals":7,"kind":kind})
        invoke(address,"mint",to=root,amount=1_000_000_000_000)
        print("funded LOCAL",symbol,flush=True)
    assets={a["symbol"]:a["address"] for a in manifest["assets"]}
    feed=deploy(base/"etesia_pricing_fixture.optimized.wasm",admin=root,usdc=assets["USDC"])
    sources=[]
    for symbol,asset in assets.items():
        if symbol=="USDC": continue
        pair=deploy(pair_wasm)
        token0,token1=sorted([asset,assets["USDC"]],key=lambda a:base64.b32decode(a)[1:33])
        invoke(pair,"initialize",factory=root,token_0=token0,token_1=token1)
        for token in [token0,token1]: invoke(token,"mint",to=pair,amount=1_000_000_000_000)
        invoke(pair,"sync")
        invoke(feed,"update",asset={"Stellar":asset},price=100_000_000_000_000,timestamp=int(time.time()))
        sources.append({"asset":asset,"decimals":7,"numerator":{"contract":feed,"asset":{"Stellar":asset},"base":{"Stellar":assets["USDC"]},"decimals":14,"resolution":300,"max_age":3600},"denominator":[],"pool":pair,"asset_is_token0":token0==asset,"min_asset_reserve":"10000000","min_usdc_reserve":"10000000","min_spacing":1,"max_gap":300,"max_age":3600,"max_skew":3600,"divergence_bps":100})
        print("upstream pair ready",symbol,flush=True)
provider_config={"usdc":assets["USDC"],"sources":sources}
provider=deploy(base/"etesia_pricing.optimized.wasm",c=provider_config)
for round_ in range(3):
    for s in sources: invoke(provider,"observe",asset=s["asset"])
    print("collected round",round_+1,flush=True)
marks=[invoke(provider,"mark",asset=s["asset"]) for s in sources]
assert all(m["ready"] and int(m["price"])==10**12 and int(m["reference"])==10**12 for m in marks)
config={"assets":sorted([{k:a[k] for k in ("address","decimals","kind")} for a in manifest["assets"]],key=lambda a:base64.b32decode(a["address"])[1:33]),"usdc":assets["USDC"],"oracle":provider,"router":feed,"route_contracts":[],"pool":None,"pool_assets":[],"admin":root,"executor":root,"guardian":root,"fees":{"management_bps":100,"performance_bps":1000,"recipient":root},"borrowing":False,"max_price_age":3600,"max_divergence_bps":100,"max_slippage_bps":100,"max_leg_bps":2000,"max_turnover_bps":10000,"max_loss_bps":200,"cooldown":1,"max_debt_bps":0,"min_health_bps":12500}
vault=deploy(base/"etesia_vault.optimized.wasm",config=config,seed_from=root)
for s in sources: invoke(s["asset"],"mint",to=vault,amount=10_000_000)
nav=int(invoke(vault,"total_assets")); assert nav==10_050_000_000
shares=int(invoke(vault,"deposit",assets=1_000_000_000,receiver=root,from_=root,operator=root))
state=invoke(vault,"state")
invoke(feed,"update",asset={"Stellar":assets["XLM"]},price=0,timestamp=int(time.time()))
try:
    invoke(vault,"redeem",shares=shares,receiver=root,owner=root,operator=root)
    raise AssertionError("invalid feed allowed redemption")
except RuntimeError:
    assert invoke(vault,"state")==state
invoke(feed,"update",asset={"Stellar":assets["XLM"]},price=100_000_000_000_000,timestamp=int(time.time()))
received=int(invoke(vault,"redeem",shares=shares,receiver=root,owner=root,operator=root))
manifest.update(result="passed",provider=provider,provider_config=provider_config,config_hash=invoke(provider,"config_hash"),vault=vault,config=config,marks=marks,nav=nav,minted_shares=shares,redeemed_assets=received,final_state=invoke(vault,"state"),upstream_pair_sha256=pair_wasm.stem,feed="controlled local ABI fixture; deployed Reflector v6 spec is incompatible with pinned CLI 22",wasm_sha256={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in base.glob("*.optimized.wasm")})
save()
print("D3 local RPC passed",nav,received,flush=True)
