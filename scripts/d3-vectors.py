#!/usr/bin/env python3
"""Independent exact rational checks for the CSVs consumed by Rust tests."""
from fractions import Fraction
from pathlib import Path
root = Path(__file__).resolve().parents[1]
count = 0
for line in (root / "fixtures/d3-ratio.csv").read_text().splitlines():
    if line.startswith("#"): continue
    n, nd, d, dd, expected = map(int, line.split(","))
    price = Fraction(n, 10**nd) / Fraction(d, 10**dd) * 10**12
    assert price.numerator // price.denominator == expected, (line, price)
    count += 1
for line in (root / "fixtures/d3-twap.csv").read_text().splitlines():
    if line.startswith("#"): continue
    a, b, c, t0, t1, t2, expected = map(int, line.split(","))
    price = (Fraction(a+b, 2)*(t1-t0)+Fraction(b+c, 2)*(t2-t1))/(t2-t0)
    assert price.numerator // price.denominator == expected, (line, price)
    count += 1
print(f"{count} exact rational vectors passed")
