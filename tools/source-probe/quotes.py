#!/usr/bin/env python3
"""Read-only actual-size Soroswap-only quotes; never transaction construction."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
import time
from urllib.error import HTTPError
from urllib.request import Request, urlopen

manifest = json.loads(Path(sys.argv[1]).read_text())
usdc = manifest["assets"]["USDC"]["id"]
rows = []
for symbol in ("XLM", "AQUA", "SHX", "USTRY"):
    for amount in (100 * 10**7, 1000 * 10**7):
        body = {"assetIn": usdc, "assetOut": manifest["assets"][symbol]["id"],
                "amount": str(amount), "tradeType": "EXACT_IN", "protocols": ["soroswap"],
                "parts": 1, "maxHops": 2}
        row = {"symbol": symbol, "captured_at": datetime.now(timezone.utc).isoformat(), "request": body}
        request = Request("https://api.soroswap.finance/quote?network=mainnet",
                          data=json.dumps(body).encode(), headers={"Content-Type": "application/json",
                          "Authorization": "Bearer " + os.environ["SOROSWAP_API_KEY"]})
        try:
            with urlopen(request, timeout=30) as response:
                result = json.load(response)
            assert result["assetIn"] == body["assetIn"] and result["assetOut"] == body["assetOut"]
            assert result["tradeType"] == "EXACT_IN" and int(result["amountIn"]) == amount
            assert int(result["amountOut"]) > 0
            assert result["routePlan"] and all(leg["swapInfo"]["protocol"] == "soroswap" for leg in result["routePlan"])
            row.update(status="quoted", amount_out=result["amountOut"], quote=result)
        except HTTPError as error:
            row.update(status="unavailable", error=f"HTTP {error.code}")
        rows.append(row)
        time.sleep(1.1)
Path(sys.argv[2]).write_text(json.dumps({"scope": "research quotes at requested sizes, not fills or NAV inputs", "quotes": rows}, indent=2) + "\n")
