#!/usr/bin/env python3
"""Record D3 local receipts and reproducible source/coverage hashes."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import urllib.request

root = Path(__file__).resolve().parents[1]
out = root / "docs/evidence"

def receipt(item):
    request = urllib.request.Request("http://127.0.0.1:8003", data=json.dumps({
        "jsonrpc": "2.0", "id": 1, "method": "getTransaction", "params": {"hash": item["hash"]}
    }).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=20) as response:
        result = json.load(response)["result"]
    assert result["status"] == "SUCCESS", result
    return {**item, "status": result["status"], "ledger": result["ledger"]}

for scenario in ("d3", "d3-conversion"):
    manifest = json.loads((root / "artifacts/local" / scenario / "manifest.json").read_text())
    assert manifest["result"] == "passed" and manifest["passphrase"] == "Standalone Network ; September 2026"
    with ThreadPoolExecutor(max_workers=6) as pool:
        manifest["transactions"] = list(pool.map(receipt, manifest["transactions"]))
    for name, digest in manifest["wasm_sha256"].items():
        assert hashlib.sha256((root / "target/wasm32-unknown-unknown/release" / name).read_bytes()).hexdigest() == digest
    manifest.pop("transactions_log")
    manifest["scope"] = "local fixture tokens/feed; actual deployed Soroswap pair WASM; no external activation"
    manifest["recorded_at"] = datetime.now(timezone.utc).isoformat()
    manifest["resource_profile_sha256"] = hashlib.sha256((root / "fixtures/local-network-settings.json").read_bytes()).hexdigest()
    (out / (scenario + "-local-rpc.json")).write_text(json.dumps(manifest, indent=2) + "\n")
    print(scenario, len(manifest["transactions"]), "successful local receipts")
coverage = json.loads((root / "coverage/coverage.json").read_text())["data"][0]
report = {"recorded_at": datetime.now(timezone.utc).isoformat(), "command": "sh scripts/check.sh",
          "lines": coverage["totals"]["lines"], "files": {str(Path(f["filename"]).relative_to(root)): f["summary"]["lines"] for f in coverage["files"]},
          "exclusions": ["vendor/", "testutils.rs", "tests.rs", "types.rs", "tools/"],
          "tools": {"rust": "1.84.1", "sdk": "22.0.8", "stellar_cli": "22.8.2", "cargo_llvm_cov": "0.6.16"}}
(out / "d3-coverage.json").write_text(json.dumps(report, indent=2) + "\n")
paths = [root / n for n in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml")]
for folder in ("contracts/pricing", "contracts/vault", "tools/pricing-fixture", "scripts", "fixtures", "tools/source-probe"):
    paths += [p for p in (root / folder).rglob("*") if p.is_file() and not {"target", "node_modules", "test_snapshots", "__pycache__"}.intersection(p.parts) and p.suffix in (".rs", ".toml", ".json", ".yaml", ".py", ".sh", ".cjs", ".csv")]
files = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
source = {"baseline": "8aa929092435ab82fccb8ae24f00bba9e4cc1387", "revision": "uncommitted D3 worktree", "files": files,
          "sha256": hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()}
(out / "d3-source-manifest.json").write_text(json.dumps(source, indent=2) + "\n")
