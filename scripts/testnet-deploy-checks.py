#!/usr/bin/env python3
"""Offline checks of the pure parts of testnet-deploy.py liquidity (A98, A99): seed amounts, the realign gate, faucet
envelope validation, the forward-and-merge operations and, with stubs, the mint and trustline evidence lines. Nothing is
read from the network; the stellar CLI only decodes."""
import base64
from datetime import UTC, datetime, timedelta
from fractions import Fraction
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("testnet_deploy", root / "scripts/testnet-deploy.py")
deploy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(deploy)  # the network check and the modes run only under __main__
count = 0


def check(name):
    global count
    count += 1
    return name


ONE_PERCENT = Fraction(1, 100)
TRADE = deploy.SEED_TRADE  # 100 USDC
# (USDC, other) reserves: the XLM pair after the 2026-11-11 17:36 realign, and the tRESERVE pair as seeded (A71).
PAIRS = {"xlm": (93_286_821_311, 618_047_135_086), "treserve": (10_050_000_000, 9_950_000_000)}

check("a 100 USDC trade needs 9,870.3 USDC of depth at 1%")
need = deploy.depth_needed(TRADE, 100)
assert need == 98_703_000_000, need
assert deploy.impact(TRADE, need) == ONE_PERCENT
assert deploy.impact(TRADE, need - 1_000_000) > ONE_PERCENT  # 9,870.2 USDC fails

check("impact matches the integer Soroswap amount_out against the mid, net of the fee")
for u, x in PAIRS.values():
    for a, r_in, r_out in ((TRADE, u, x), (TRADE * x // u, x, u)):
        out = a * 997 * r_out // (r_in * 1000 + a * 997)
        from_out = 1 - Fraction(out * r_in, a * r_out) / deploy.GAMMA
        assert abs(from_out - deploy.impact(a, r_in)) < Fraction(1, 10**8), (a, r_in, from_out)

check("seed adds keep the mid, reach the depth and pass both directions; 9,870.2 would not")
for which, (u, x) in PAIRS.items():
    add_u, add_x = deploy.seed_adds(u, x, need)
    taken = add_u * x // u  # the router's optimal amount of the other token
    assert add_x - 1 <= taken <= add_x, (which, add_x, taken)
    u2, x2 = u + add_u, x + taken
    assert u2 == need
    assert abs(Fraction(x2) - Fraction(u2 * x, u)) < 1, which  # mid within one unit
    assert deploy.impact(TRADE, u2) <= ONE_PERCENT  # buy with 100 USDC
    assert deploy.impact(Fraction(TRADE * x2, u2), x2) <= ONE_PERCENT  # sell 100 USDC worth at the mid
    short_u, short_x = deploy.seed_adds(u, x, need - 1_000_000)
    assert deploy.impact(TRADE, u + short_u) > ONE_PERCENT
    assert deploy.minimum(add_u) == add_u * 9_990 // 10_000 and deploy.minimum(add_x) <= taken
assert deploy.seed_adds(need, 654_000_000_000, need) is None and deploy.seed_adds(need + 1, 1, need) is None

check("seed refuses without fresh included realigns of both pairs")
now = datetime(2026, 11, 12, 12, 0, tzinfo=UTC)


def line(pair, status, age):
    return {"pair": pair, "status": status, "written_at": (now - timedelta(seconds=age)).isoformat()}


assert set(deploy.realigned([line("xlm", "included", 30), line("treserve", "no trade", 60)], now)) == {"xlm", "treserve"}
assert set(deploy.realigned([line("xlm", "included", 30), line("treserve", "included", deploy.REALIGN_FRESH + 1)], now)) == {"xlm"}
assert set(deploy.realigned([line("xlm", "included", 30), line("xlm", "signed", 20), line("treserve", "included", 30)], now)) == {"treserve"}
assert deploy.realigned([], now) == {}

check("faucet envelope quotes are stripped")
assert deploy.strip_envelope('"AAAAAgAA+/="\n') == "AAAAAgAA+/=" and deploy.strip_envelope("AAAAAgAA") == "AAAAAgAA"

check("the archived A81 claim envelope validates; foreign sources, fee payers and operations do not")
issuer = deploy.inputs["assets"]["USDC"]["classic"].split(":")[1]
archived = json.loads((root / "docs/evidence/t2-faucet-a81.json").read_text())["transactions"][1]["envelopeXdr"]
decoded = json.loads(subprocess.run(["stellar", "tx", "decode", archived, "--output", "json"], text=True, capture_output=True, check=True).stdout)
ops = decoded["tx"]["tx"]["operations"]
fresh = ops[0]["source_account"]
assert len(ops) == 8 and deploy.claim_problems(decoded, fresh, issuer) == [], deploy.claim_problems(decoded, fresh, issuer)
other = "GDRVYVG3SQC4DJ6L7DPPGV6OQSBCZZWJA64VDXUJDQVMWN2A4KRXZM3R"


def edited(change):
    copy = json.loads(json.dumps(decoded))
    change(copy["tx"]["tx"])
    return deploy.claim_problems(copy, fresh, issuer)


assert deploy.claim_problems(decoded, other, issuer)  # the envelope is for another claimant
assert deploy.claim_problems({"tx_fee_bump": decoded["tx"]}, fresh, issuer)
assert edited(lambda tx: tx.update(source_account=other))  # another fee payer
assert edited(lambda tx: tx["operations"].append({"source_account": fresh, "body": {"account_merge": other}}))
assert edited(lambda tx: tx["operations"].append({"source_account": fresh, "body": "inflation"}))
assert edited(lambda tx: tx["operations"][0].update(source_account=other))  # change_trust by someone else
assert edited(lambda tx: tx["operations"][1]["body"]["payment"].update(destination=other))
assert edited(lambda tx: tx["operations"][1].update(source_account=fresh))  # a payment out of the claimant
assert edited(lambda tx: tx["operations"][3]["body"]["payment"]["asset"]["credit_alphanum4"].update(issuer=other))
assert edited(lambda tx: tx["operations"][2]["body"]["change_trust"]["line"]["credit_alphanum4"].update(asset_code="EURC"))
assert edited(lambda tx: tx.update(operations=tx["operations"][2:])) == ["no USDC payment to the claimant"]

check("tx_bad_seq is read from the RPC errorResultXdr")
result = lambda code: {"status": "ERROR", "errorResultXdr": base64.b64encode((100).to_bytes(8, "big") + code.to_bytes(4, "big", signed=True) + bytes(4)).decode()}
assert deploy.bad_seq(result(-5)) and not deploy.bad_seq(result(-1)) and not deploy.bad_seq({"status": "PENDING"})

check("forward-and-merge: USDC to etesia-liquidity, the rest back to the issuer, trustlines removed, merge last")
liquidity = "GCB3XCLIQUIDITYPLACEHOLDERXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
assert deploy.merge_ops(liquidity, issuer, {"USDC": 10_000_000_000, "BLND": 50_000_000_000, "wETH": 5_000_000, "wBTC": 0}) == [
    ["payment", "--destination", liquidity, "--asset", f"USDC:{issuer}", "--amount", "10000000000"],
    ["payment", "--destination", issuer, "--asset", f"BLND:{issuer}", "--amount", "50000000000"],
    ["payment", "--destination", issuer, "--asset", f"wETH:{issuer}", "--amount", "5000000"],
    *[["change-trust", "--line", f"{code}:{issuer}", "--limit", "0"] for code in ("USDC", "BLND", "wETH", "wBTC")],
    ["account-merge", "--account", liquidity]]
assert deploy.merge_ops(liquidity, issuer, {}) == [["account-merge", "--account", liquidity]]

# Stubs from here on: nothing is sent, the state and liquidity log are scratch files.
scratch = Path(tempfile.mkdtemp())
deploy.STATE, deploy.LIQUIDITY_LOG = scratch / "state.json", scratch / "liquidity.jsonl"
deploy.fetch = lambda *args: []
deploy.address = lambda name: "G" + name
deploy.included = lambda hashes: [{"hash": h, "ledger": 1, "created_at": "t"} for h in hashes]
H = "ab" * 32


def rerun_after_cli_failure(mode, key, sends):
    """`mode` once with a CLI that fails after signing, then again: `sends` CLI sends in all, the archived lines of `H`."""
    deploy.state.clear()
    deploy.state.update({"treserve_sac": {"asset": f"TRES:{issuer}"}, "identities": {deploy.LIQUIDITY: "GLIQ"}})
    deploy.LIQUIDITY_LOG.unlink(missing_ok=True)
    calls = []

    def command(args):
        calls.append(args)
        if calls == [args]:
            raise deploy.CliError(f"Signing transaction: {H}\nerror: timed out waiting for the transaction", [H])
        return "", ["cd" * 32]
    deploy.command = command
    deploy.fetch = lambda *args: [{"trustline": True}] if mode is deploy.liquidity_mint else []
    for _ in (1, 2):
        try:
            mode(True)
        except deploy.CliError:
            assert deploy.state[key]["hashes"] == [H]
    assert len(calls) == sends, calls
    return [line for line in map(json.loads, deploy.LIQUIDITY_LOG.read_text().splitlines()) if line.get("hashes") == [H]]


check("a mint whose CLI failed after signing is archived as signed, then included on the rerun")
lines = rerun_after_cli_failure(deploy.liquidity_mint, "liquidity_mint", 1)
assert [line["status"] for line in lines] == ["signed", "included"], lines
assert lines[1]["amount"] == deploy.MINT and lines[1]["transactions"][0]["hash"] == H

check("a trustline whose CLI failed after signing is archived as signed, then included on the rerun")
deploy.identities = lambda names, key: None
lines = rerun_after_cli_failure(deploy.liquidity_account, "liquidity_usdc_trustline", 2)  # then the tRESERVE one
assert [line["status"] for line in lines] == ["signed", "included"], lines
assert lines[1]["line"] == deploy.inputs["assets"]["USDC"]["classic"]

check("a mint that never landed is archived as signed before the step is forgotten; a rerun archives included once")
deploy.state.clear()
deploy.state.update({"treserve_sac": {"asset": f"TRES:{issuer}"}, "identities": {deploy.LIQUIDITY: "GLIQ"}})
deploy.LIQUIDITY_LOG.unlink(missing_ok=True)
deploy.command = lambda args: ("", [H])
deploy.fetch = lambda *args: [{"trustline": True}]
deploy.included = lambda hashes: None
try:
    deploy.liquidity_mint(True)
except SystemExit:
    pass
assert "liquidity_mint" not in deploy.state
deploy.included = lambda hashes: [{"hash": h, "ledger": 1, "created_at": "t"} for h in hashes]
deploy.liquidity_mint(True)
deploy.liquidity_mint(True)  # recorded: nothing new to archive
assert [json.loads(line)["status"] for line in deploy.LIQUIDITY_LOG.read_text().splitlines()] == ["signed", "signed", "included"]

print(f"{count} testnet-deploy liquidity checks passed")
