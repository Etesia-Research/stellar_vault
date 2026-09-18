#!/usr/bin/env python3
"""One collection pass; scheduling and funded transaction operation belong to D4."""
import argparse
import json
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("--network", required=True, help="Configured Stellar CLI network")
p.add_argument("--source", required=True, help="Configured funded CLI identity")
p.add_argument("--provider", required=True)
p.add_argument("--config-hash", required=True, help="Reviewed provider configuration hash")
a = p.parse_args()
base = ["stellar", "contract", "invoke", "--network", a.network, "--source", a.source, "--id", a.provider]

def invoke(method, *args, send="no"):
    result = subprocess.run([*base, "--send=" + send, "--", method, *args], text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    return json.loads(result.stdout)

if invoke("config_hash") != a.config_hash:
    raise SystemExit("Provider configuration differs from the reviewed hash")
config = invoke("config")
for source in config["sources"]:
    # The contract chooses the pool, reserves, price, ledger and time.
    observation = invoke("observe", "--asset", source["asset"], send="yes")
    print(json.dumps({"asset": source["asset"], "observation": observation}), flush=True)
