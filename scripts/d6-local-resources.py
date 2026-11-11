#!/usr/bin/env python3
"""D6-02: worst-case `execute` resources on the isolated local protocol-30 network.

Local RPC only (`scripts/local-node.py` running). SAC assets issued by the public standalone
genesis account, the mainnet Soroswap code (fixtures/wasm/SHA256SUMS), the Etesia pricing provider
over a locally configured copy of the testnet Reflector with a USDC denominator leg (the testnet
provider's two-feed shape), and the testnet Blend v2 pool, backstop and Comet. Those four WASMs are
passed as arguments. Every scenario is a real local `execute`, sent after its simulation passes.
Writes docs/evidence/d6-02-local-resources.json, or d6-02-local-resources-raised.json when the node
runs fixtures/d6-02-raised-limits.json: then every plan applies and its receipt shows what it needs.
The record is rewritten after every scenario, so a run that stops keeps its receipts so far.
Routes have at most 2 hops and at most 2 routes per swap: the keeper sends 1 hop and 1 route
(etesia-quant routing.py), the mainnet API quote allows 2 hops (api.py). The contract's maximum of
4 hops and 15 routes per swap is not measured.

usage: d6-local-resources.py <reflector.wasm> <blend pool.wasm> <blend backstop.wasm> <comet.wasm> <node data directory>
"""
import base64
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
os.environ.setdefault("STELLAR_CONFIG_HOME", str(ROOT / ".stellar"))
RPC = "http://127.0.0.1:8003"
PASSPHRASE = "Standalone Network ; September 2026"
REFLECTOR_WASM, POOL_WASM, BACKSTOP_WASM, COMET_WASM, DATA = (Path(p).resolve() for p in sys.argv[1:6])
# Headroom is always against the protocol-30 profile; a node started with the raised profile measures what a plan needs.
P30 = {k: v for e in json.loads((ROOT / "fixtures/local-network-settings.json").read_text())["updated_entry"] for k, v in e.items()}
LIMITS = json.loads((DATA / "resource-limits.json").read_text())
# The data directory must belong to the node serving RPC: its limits decide the output file.
assert LIMITS["tx"] == json.loads(urllib.request.urlopen("http://127.0.0.1:11626/sorobaninfo", timeout=5).read())["tx"], \
    "The last argument is not the running node's data directory"
RAISED = LIMITS["tx"]["memory_limit"] > P30["contract_compute_v0"]["tx_memory_limit"]
OUT = ROOT / ("docs/evidence/d6-02-local-resources-raised.json" if RAISED else "docs/evidence/d6-02-local-resources.json")
RELEASE = ROOT / "target/wasm32v1-none/release"
FIXTURES = ROOT / "fixtures/wasm"
PROTOCOL_BPS = 10
DEPTH = 1_000_000_000_000_000  # 100M whole units a side: repeated legs move no pair enough to fail the 1% floor
LEG = 5_000_000_000  # 500 USDC, 0.5% of the 100k-USDC vault
METRICS = ("cpu_insn", "mem_byte", "read_entry", "write_entry", "ledger_read_byte", "ledger_write_byte",
           "read_key_byte", "write_key_byte", "read_data_byte", "write_data_byte", "read_code_byte",
           "write_code_byte", "emit_event", "emit_event_byte", "max_rw_key_byte", "max_rw_data_byte",
           "max_rw_code_byte", "max_emit_event_byte", "invoke_time_nsecs")
record = {"captured_at": datetime.now(UTC).isoformat(timespec="seconds"), "network": "local-only", "passphrase": PASSPHRASE, "rpc": RPC, "protocol": 30,
          "revisions": {"vault": {"commit": subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, capture_output=True).stdout.strip(),
                                  "dirty": bool(subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, text=True, capture_output=True).stdout)},
                        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()},
          "tools": {"stellar_cli": subprocess.run(["stellar", "--version"], text=True, capture_output=True).stdout.splitlines()[0]},
          "label": "Isolated local protocol-30 network; not Stellar Testnet or Mainnet",
          "node_limits": LIMITS, "raised_limits": RAISED, "scenarios": []}


def save():
    OUT.write_text(json.dumps(record, indent=1) + "\n")


def command(args):
    result = subprocess.run(["stellar", *args], text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(" ".join(args[:6]) + "\n" + result.stderr)
    return result.stdout.strip()


def rpc(method, **params):
    request = urllib.request.Request(RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(),
                                     headers={"Content-Type": "application/json"})
    response = json.load(urllib.request.urlopen(request, timeout=30))
    if "error" in response:
        raise RuntimeError(response["error"])
    return response["result"]


def arguments(kwargs):
    args = []
    for key, value in kwargs.items():
        args += ["--" + key.removesuffix("_").replace("_", "-"),
                 json.dumps(value, separators=(",", ":")) if isinstance(value, (dict, list)) else str(value)]
    return args


def invoke(contract, method, **kwargs):
    out = command(["contract", "invoke", "--id", contract, "--source", "local-root", "--network", "local", "--", method, *arguments(kwargs)])
    return json.loads(out) if out else None


def deploy(wasm, **kwargs):
    return command(["contract", "deploy", "--wasm", str(wasm), "--source", "local-root", "--network", "local", "--", *arguments(kwargs)]).splitlines()[-1]


def now():
    return int(rpc("getLedgers", startLedger=rpc("getLatestLedger")["sequence"], pagination={"limit": 1})["ledgers"][0]["ledgerCloseTime"])


def metrics(diagnostic):
    # The host's `core_metrics` diagnostic events: topic 1 names the metric, data is its u64 value.
    found = {}
    for event in diagnostic:
        body = event["event"]["body"]["v0"]
        if body["topics"][:1] == [{"symbol": "core_metrics"}]:
            found[body["topics"][1]["symbol"]] = int(body["data"]["u64"])
    return {k: found[k] for k in METRICS}


def calls(diagnostic):
    # Contract invocations by function name (`fn_call` diagnostic events): each one instantiates a VM.
    count = {}
    for event in diagnostic:
        topics = event["event"]["body"]["v0"]["topics"]
        if topics[:1] == [{"symbol": "fn_call"}]:
            count[topics[2]["symbol"]] = count.get(topics[2]["symbol"], 0) + 1
    return dict(sorted(count.items(), key=lambda kv: -kv[1]))


def resources(data):
    r = data["resources"]
    return {"instructions": r["instructions"], "disk_read_bytes": r["disk_read_bytes"], "write_bytes": r["write_bytes"],
            "footprint_read_only": len(r["footprint"]["read_only"]), "footprint_read_write": len(r["footprint"]["read_write"]),
            "resource_fee": int(data["resource_fee"])}


def receipt(tx_hash):
    tx = rpc("getTransaction", hash=tx_hash, xdrFormat="json")
    assert tx["status"] in ("SUCCESS", "FAILED"), tx
    raw = rpc("getTransaction", hash=tx_hash)
    v1 = tx["envelopeJson"]["tx"]["tx"]
    meta = ((tx["resultMetaJson"]["v4"].get("soroban_meta") or {}).get("ext") or {}).get("v1") or {}
    return {"hash": tx_hash, "status": tx["status"], "ledger": tx["ledger"], "envelope_bytes": len(base64.b64decode(raw["envelopeXdr"])),
            "fee_bid": v1["fee"], "fee_charged": int(tx["resultJson"]["fee_charged"]),
            "declared": resources(v1["ext"]["v1"]),
            "charged": {k: int(meta[k]) for k in ("total_non_refundable_resource_fee_charged", "total_refundable_resource_fee_charged", "rent_fee_charged") if k in meta},
            "core_metrics": metrics(tx["diagnosticEventsJson"]), "calls": calls(tx["diagnosticEventsJson"])}


def headroom(**used):
    """Each value against the protocol-30 per-transaction limit (fixtures/local-network-settings.json, as on live testnet)."""
    limits = {"instructions": ("contract_compute_v0", "tx_max_instructions"), "memory": ("contract_compute_v0", "tx_memory_limit"),
              "read_entries": ("contract_ledger_cost_v0", "tx_max_disk_read_entries"),
              "write_entries": ("contract_ledger_cost_v0", "tx_max_write_ledger_entries"),
              "footprint": ("contract_ledger_cost_ext_v0", "tx_max_footprint_entries"),
              "read_bytes": ("contract_ledger_cost_v0", "tx_max_disk_read_bytes"), "write_bytes": ("contract_ledger_cost_v0", "tx_max_write_bytes"),
              "events_bytes": ("contract_events_v0", "tx_max_contract_events_size_bytes"), "tx_size": ("contract_bandwidth_v0", "tx_max_size_bytes")}
    out = {}
    for k, u in used.items():
        limit = int(P30[limits[k][0]][limits[k][1]])
        out[k] = {"used": u, "limit": limit, "pct": round(100 * u / limit, 1)}
    return out


network = rpc("getNetwork")
assert network["passphrase"] == PASSPHRASE, "Refuse a non-local network"
record["tools"]["rpc"] = rpc("getVersionInfo")
root = command(["keys", "address", "local-root"])
subprocess.run(["sha256sum", "--quiet", "-c", "SHA256SUMS"], cwd=FIXTURES, check=True)
# (1) Six SAC assets: classic credit assets issued by the genesis account, and native XLM.
universe = [("USDC", "Settlement"), ("XLM", "Xlm"), ("AQUA", "Risk"), ("ETH", "Risk"), ("BTC", "Risk"), ("USTRY", "Reserve")]
assets = {}
for symbol, _ in universe:
    asset = "native" if symbol == "XLM" else f"{symbol}:{root}"
    assets[symbol] = command(["contract", "asset", "deploy", "--asset", asset, "--source", "local-root", "--network", "local"]).splitlines()[-1]
usdc = assets["USDC"]
# (2) Pricing: the Etesia provider over the Reflector code, quoted in USD like the testnet External feed; every
# non-USDC source has a USDC denominator leg. One price round (milliseconds, on the 300 s grid) at par for all six.
feed = deploy(REFLECTOR_WASM)
invoke(feed, "config", config={"admin": root, "assets": [{"Stellar": a} for a in assets.values()], "base_asset": {"Other": "USD"},
                               "cache_size": 2, "decimals": 14, "fee_config": "None", "history_retention_period": 86_400_000, "resolution": 300_000})
assert invoke(feed, "resolution") == 300, "Reflector config takes milliseconds and reports seconds"
invoke(feed, "set_price", updates={"mask": format(2 ** len(assets) - 1, "02x"), "prices": [str(10**14)] * len(assets)},
       timestamp=now() // 300 * 300 * 1000)


def leg(address):
    return {"contract": feed, "asset": {"Stellar": address}, "base": {"Other": "USD"}, "decimals": 14, "resolution": 300, "max_age": 3600}


provider_config = {"usdc": usdc, "sources": [{"asset": a, "numerator": leg(a), "denominator": [leg(usdc)]} for s, a in assets.items() if s != "USDC"]}
provider = deploy(RELEASE / "etesia_pricing.wasm", c=provider_config)
assert all(int(invoke(provider, "mark", asset=a)["price"]) == 10**12 for a in assets.values())
# One mark's simulated instructions, to compare with the testnet provider over the live Reflector (same shape).
xdr = command(["contract", "invoke", "--id", provider, "--source", "local-root", "--network", "local", "--build-only", "--", "mark", "--asset", assets["XLM"]])
record["mark_simulated_instructions"] = rpc("simulateTransaction", transaction=xdr, xdrFormat="json")["transactionDataJson"]["resources"]["instructions"]
# (3) Soroswap: mainnet code, all 15 pairs of the universe seeded at parity.
pair_hash = command(["contract", "upload", "--wasm", str(FIXTURES / "soroswap_pair.wasm"), "--source", "local-root", "--network", "local"]).splitlines()[-1]
factory = deploy(FIXTURES / "soroswap_factory.wasm")
invoke(factory, "initialize", setter=root, pair_wasm_hash=pair_hash)
soroswap = deploy(FIXTURES / "soroswap_router.wasm")
invoke(soroswap, "initialize", factory=factory)
# The issuer as v3 treasury: a fee paid to it needs no trustline. The Aqua router is a placeholder the vault never reaches.
router = deploy(FIXTURES / "soroswap_aggregator_v3.wasm", admin=root, treasury=root, protocol_bps=PROTOCOL_BPS,
                min_partner_bps=PROTOCOL_BPS, soroswap_router=soroswap, aqua_router=root)
symbols = [s for s, _ in universe]
pairs = {}
for i, a in enumerate(symbols):
    for b in symbols[i + 1:]:
        invoke(soroswap, "add_liquidity", token_a=assets[a], token_b=assets[b], amount_a_desired=DEPTH, amount_b_desired=DEPTH,
               amount_a_min=0, amount_b_min=0, to=root, deadline=now() + 3600)
        pairs[frozenset((a, b))] = invoke(factory, "get_pair", token_a=assets[a], token_b=assets[b])
print("pairs", len(pairs), flush=True)
# (4) Blend v2: the testnet pool and backstop code, USDC reserve with the testnet pool's USDC config.
blnd = command(["contract", "asset", "deploy", "--asset", f"BLND:{root}", "--source", "local-root", "--network", "local"]).splitlines()[-1]
# The backstop values its token through the Comet BLND:USDC 80:20 LP (testnet Comet code); it holds no deposits here.
comet = deploy(COMET_WASM)
invoke(comet, "init", controller=root, tokens=[blnd, usdc], weights=["8000000", "2000000"],
       balances=["10000000000000", "2500000000000"], swap_fee="30000")
backstop = deploy(BACKSTOP_WASM, backstop_token=comet, emitter=root, blnd_token=blnd, usdc_token=usdc, pool_factory=root, drop_list=[])
pool = deploy(POOL_WASM, admin=root, name="LocalV2", oracle=feed, bstop_rate=1_000_000, max_positions=8, min_collateral=0,
              backstop_id=backstop, blnd_id=blnd)
invoke(pool, "queue_set_reserve", asset=usdc, metadata={
    "index": 0, "decimals": 7, "c_factor": 9_500_000, "l_factor": 9_500_000, "util": 7_000_000, "max_util": 9_500_000,
    "r_base": 5_000, "r_one": 300_000, "r_two": 1_000_000, "r_three": 10_000_000, "reactivity": 20,
    "supply_cap": "170141183460469231731687303715884105727", "enabled": True})
invoke(pool, "set_reserve", asset=usdc)
invoke(pool, "set_status", pool_status=2)  # admin on-ice: Supply and Withdraw allowed, borrowing not
print("blend pool status", invoke(pool, "get_config")["status"], flush=True)
# (5) The vault: testnet flagship limits (cooldown 1 for one-second ledgers), all 15 pairs, the Blend USDC reserve.
config = {"assets": sorted([{"address": assets[s], "decimals": 7, "kind": k} for s, k in universe], key=lambda a: base64.b32decode(a["address"])[1:33]),
          "usdc": usdc, "oracle": provider, "router": router, "route_contracts": sorted(pairs.values()), "pool": pool, "pool_assets": [usdc],
          "admin": root, "executor": root, "guardian": root, "fees": {"management_bps": 100, "performance_bps": 1000, "recipient": root},
          "borrowing": False, "max_price_age": 3600, "max_slippage_bps": 100, "max_leg_bps": 2000, "max_turnover_bps": 10000,
          "max_loss_bps": 200, "cooldown": 1, "max_debt_bps": 0, "min_health_bps": 12500}
vault = deploy(RELEASE / "etesia_vault.wasm", config=config, seed_from=root)
invoke(vault, "deposit", assets=990_000_000_000, receiver=root, from_=root, operator=root)  # 99,000 + the 1,000 seed
print("vault", vault, flush=True)


def swap(token_in, token_out, routes):
    """`routes` lists the intermediate symbols of each route: [] is direct. The post-fee amount is split evenly."""
    net = LEG - LEG * PROTOCOL_BPS // 10_000
    split = [net // len(routes)] * len(routes)
    split[-1] += net - sum(split)
    built = []
    for amount, via in zip(split, routes):
        path = [token_in, *via, token_out]
        built.append({"amount_in": str(amount), "hops": [{"Soroswap": {"pair": pairs[frozenset((a, b))], "token_in": assets[a], "token_out": assets[b]}}
                                                         for a, b in zip(path, path[1:])]})
    # The vault floor at par marks: ceil(LEG * (10^4 - max_slippage_bps) / 10^4).
    return {"Swap": {"token_in": assets[token_in], "token_out": assets[token_out], "amount": str(LEG),
                     "minimum": str(-(-LEG * (10_000 - config["max_slippage_bps"]) // 10_000)), "routes": built}}


def publish(weights, yield_bps):
    """Publish a target with `weights` by symbol; returns its hash, the target and the vault state."""
    state = invoke(vault, "state")
    epoch = publish.epoch = getattr(publish, "epoch", 0) + 1
    by_address = {assets[s]: w for s, w in weights.items()}
    target = {"network": hashlib.sha256(PASSPHRASE.encode()).hexdigest(), "vault": vault, "epoch": epoch, "expiry": now() + 3600,
              "model": "01" * 32, "dataset": "02" * 32, "holdings": invoke(vault, "holdings_hash"),
              "assets": [a["address"] for a in config["assets"]], "weights": [by_address[a["address"]] for a in config["assets"]],
              "yield_bps": yield_bps, "equity": str(invoke(vault, "total_assets")), "supply": str(state["supply"])}
    return invoke(vault, "publish_target", target=target), target, state


def plan(target_hash, target, state, actions):
    return {"target": target_hash, "epoch": target["epoch"], "nonce": state["nonce"], "version": state["version"],
            "deadline": target["expiry"], "actions": actions}


def simulate(xdr):
    """The RPC's own simulation, retried on a non-budget error (a transient snapshot error was seen once).
    RPC 30.0.0 returns no `cost`, so simulation memory is not observable; `instructions` is the simulated
    figure after the RPC's adjustment."""
    for _ in range(3):
        sim = rpc("simulateTransaction", transaction=xdr, xdrFormat="json")
        if "error" not in sim or "Error(Budget, ExceededLimit)" in sim["error"]:
            break
    out = {"error": sim["error"][:600]} if "error" in sim else {"instructions": sim["transactionDataJson"]["resources"]["instructions"],
                                                                 "min_resource_fee": int(sim["minResourceFee"])}
    return out, sim


def execute(name, shape, weights, yield_bps, actions):
    """One sent execute; the CLI sends only after a successful simulation. Returns whether it applied."""
    target_hash, target, state = publish(weights, yield_bps)
    p = plan(target_hash, target, state, actions)
    xdr = command(["contract", "invoke", "--id", vault, "--source", "local-root", "--network", "local", "--build-only", "--",
                   "execute", "--plan", json.dumps(p, separators=(",", ":"))])
    simulation, sim = simulate(xdr)
    result = subprocess.run(["stellar", "contract", "invoke", "--id", vault, "--source", "local-root", "--network", "local", "--",
                             "execute", "--plan", json.dumps(p, separators=(",", ":"))], text=True, capture_output=True)
    hashes = re.findall(r"(?:Signing transaction|Transaction hash):\s*([0-9a-f]{64})", result.stderr)
    if not hashes:  # never sent: only a budget failure is a measurement, any other error means the plan is wrong
        assert "Error(Budget, ExceededLimit)" in result.stderr, name + "\n" + result.stderr
        assert "Error(Budget, ExceededLimit)" in simulation.get("error", ""), name + "\n" + str(simulation)
        record["scenarios"].append({"name": name, "shape": shape, "actions": len(actions), "status": "SIMULATION_EXCEEDED_BUDGET",
                                    "cli_error": [line for line in result.stderr.splitlines() if "rror" in line][:3],
                                    "rpc_error": simulation["error"],
                                    "last_events": [json.dumps(e)[:300] for e in sim.get("eventsJson", [])[-4:]]})
        save()
        print(name, "SIMULATION_EXCEEDED_BUDGET", flush=True)
        return False
    r = receipt(hashes[-1])
    assert (r["status"] == "SUCCESS") == (invoke(vault, "state")["nonce"] == state["nonce"] + 1) == (result.returncode == 0)
    m = r["core_metrics"]
    r.update(name=name, shape=shape, actions=len(actions), simulation=simulation, cli_error=[line for line in result.stderr.splitlines() if "rror" in line][:3],
             headroom=headroom(instructions=m["cpu_insn"], memory=m["mem_byte"], read_entries=m["read_entry"], write_entries=m["write_entry"],
                               footprint=r["declared"]["footprint_read_only"] + r["declared"]["footprint_read_write"],
                               read_bytes=m["ledger_read_byte"], write_bytes=m["ledger_write_byte"], events_bytes=m["emit_event_byte"],
                               tx_size=r["envelope_bytes"]))
    record["scenarios"].append(r)
    save()
    print(name, r["status"], m["cpu_insn"], m["emit_event_byte"], r["fee_charged"], flush=True)
    return r["status"] == "SUCCESS"


def largest(prefix, shape, weights, yield_bps, build, counts):
    """Send `build(n)` for each n in `counts`, largest first, until one applies."""
    for n in counts:
        if execute(f"{prefix}-{n}", shape.format(n=n), weights, yield_bps, build(n)):
            return n


# Holdings start at 100% USDC. The target is chosen so every plan's buys reduce distance (execution.rs).
# yield_bps 1,000 needs settlement - 250 == (reserve + settlement - 250) / 10 (validate_target): 750 and 4,500.
YIELD = {"USDC": 750, "XLM": 1250, "AQUA": 1250, "ETH": 1250, "BTC": 1000, "USTRY": 4500}
ONE = [("USDC", x, [[]]) for x in ("XLM", "AQUA", "ETH", "BTC", "USTRY", "XLM", "AQUA", "ETH")]
# At most 2 hops and 2 routes per swap; the contract allows 4 and 15 (execution.rs `swap`), which is not measured.
TWO = [("USDC", x, [["XLM"]]) for x in ("BTC", "USTRY", "AQUA", "ETH", "BTC", "USTRY", "AQUA", "ETH")]
SPLIT = [("USDC", x, [[], ["XLM"]]) for _, x, _ in TWO]
supply, withdraw = {"Supply": [usdc, str(LEG)]}, {"Withdraw": [usdc, str(LEG // 2)]}
# Each valuation prices every asset the vault holds, so cost grows with holdings. Fill them one leg at a time
# (a Blend supply, then one asset per execute) until a single-leg execute no longer fits.
assert execute("W0", "warm-up: Blend Supply", YIELD, 1000, [supply])
record["held_non_usdc_assets"] = []  # XLM counts: valuation prices every nonzero holding, whatever its kind
for i, s in enumerate(ONE[:5]):
    if not execute(f"W{i + 1}", f"one-hop swap to {s[1]}, holding {i} non-USDC assets", YIELD, 1000, [swap(*s)]):
        break
    record["held_non_usdc_assets"].append(s[1])
record["largest"] = {
    "one_hop": largest("S1", "{n} one-hop swaps", YIELD, 1000, lambda n: [swap(*s) for s in ONE[:n]], range(8, 0, -1)),
    "two_hop": largest("S2", "{n} two-hop swaps (via XLM)", YIELD, 1000, lambda n: [swap(*s) for s in TWO[:n]], range(8, 0, -1)),
    "two_hop_supply": largest("S3", "{n} two-hop swaps + Blend Supply", YIELD, 1000,
                              lambda n: [swap(*s) for s in TWO[:n]] + [supply], range(7, -1, -1)),
    # The planner's shape (planner.py: Withdraw, sells and buys, the last slot for Supply) at the mainnet API quote's maxHops 2 (api.py).
    "withdraw_two_hop_supply": largest("S4", "Blend Withdraw + {n} two-hop swaps + Blend Supply", YIELD, 1000,
                                       lambda n: [withdraw] + [swap(*s) for s in TWO[:n]] + [supply], range(6, -1, -1)),
    "split_two_routes": largest("S5", "{n} swaps split over a direct and a via-XLM route", YIELD, 1000,
                                lambda n: [swap(*s) for s in SPLIT[:n]], range(8, 0, -1)),
    # The planner's shape at the testnet keeper's maxHops 1 (routing.py).
    "withdraw_one_hop_supply": largest("S6", "Blend Withdraw + {n} one-hop swaps + Blend Supply", YIELD, 1000,
                                       lambda n: [withdraw] + [swap(*s) for s in ONE[:n]] + [supply], range(6, -1, -1)),
    # The testnet planner's shape (D4-05 receipt): one-hop buys, then Supply.
    "one_hop_supply": largest("S7", "{n} one-hop swaps + Blend Supply", YIELD, 1000,
                              lambda n: [swap(*s) for s in ONE[:n]] + [supply], range(7, -1, -1)),
}
# The proposed limits-v1 keeper caps, each sent once: they must apply with margin.
record["caps"] = {name: execute(name, shape, YIELD, 1000, actions) for name, shape, actions in (
    ("C1", "5 one-hop swaps", [swap(*s) for s in ONE[:5]]),
    ("C2", "3 two-hop swaps", [swap(*s) for s in TWO[:3]]),
    ("C3", "Blend Withdraw + 3 one-hop swaps + Blend Supply", [withdraw] + [swap(*s) for s in ONE[:3]] + [supply]),
    ("C4", "Blend Withdraw + 2 two-hop swaps + Blend Supply", [withdraw] + [swap(*s) for s in TWO[:2]] + [supply]))}
# Where a buy stopped fitting, add the remaining assets by direct SAC transfer instead, and try one
# one-hop buy of XLM (already held) at each holdings count.
record["transferred_holdings"] = {}
for _, symbol, _ in ONE[len(record["held_non_usdc_assets"]):5]:
    invoke(assets[symbol], "transfer", from_=root, to=vault, amount=LEG)
    held = len(record["held_non_usdc_assets"]) + len(record["transferred_holdings"]) + 1
    record["transferred_holdings"][symbol] = execute(f"T{held}", f"one-hop swap to XLM, holding {held} non-USDC assets ({symbol} transferred in)",
                                                     YIELD, 1000, [swap(*ONE[0])])
record.update(vault=vault, provider=provider, feed=feed, router=router, factory=factory, soroswap_router=soroswap, pool=pool, backstop=backstop, comet=comet,
              config=config, provider_config=provider_config, pairs={"/".join(sorted(k)): v for k, v in pairs.items()},
              wasm_sha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in
                           [RELEASE / "etesia_vault.wasm", RELEASE / "etesia_pricing.wasm", *FIXTURES.glob("*.wasm"),
                            REFLECTOR_WASM, POOL_WASM, BACKSTOP_WASM, COMET_WASM]})
save()
print("wrote", OUT.relative_to(ROOT), flush=True)
