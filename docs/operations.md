# Direct operations and recovery

Local D2 contract behavior, verified 2026-09-18. Live identities and limits must
come from a separately accepted deployment manifest. No Etesia keeper is required
for the operations below. Use the [local RPC guide](local-rpc.md) to reproduce the
fixture deployment; never use its price feeds or known keys on another network.

## Direct USDC exit

1. Read `config`, `state`, `holdings`, `max_withdraw(owner)` and
   `preview_redeem(shares)`. Share units are 12 decimals; USDC units are 7.
2. If fees are more than one year behind, call `checkpoint_fees()` repeatedly
   until `last_fee` is within one year of ledger time. Each call preserves elapsed
   entitlement and requires no role authorization.
3. With healthy pricing and enough USDC, sign a bounded exit directly:

```sh
stellar contract invoke --id "$VAULT" --source "$OWNER_KEY" --network "$NETWORK" -- \
  redeem_bounded --shares "$SHARES" --receiver "$OWNER_ADDRESS" \
  --owner "$OWNER_ADDRESS" --operator "$OWNER_ADDRESS" \
  --min-assets "$MIN_USDC_ATOMS" --deadline "$DEADLINE"
```

The owner supplies these variables from their reviewed transaction. A failed
call leaves ownership and custody unchanged. Signing as a delegated operator
requires an unexpired share allowance. Never infer settlement from a simulation.

## Restore liquidity without the executor

Anyone can call `unwind(actions, nonce, deadline)` with a reviewed Soroswap quote
and exact bounded auth tree. Read the current nonce first. Only withdrawals,
repayments, collateral release and swaps toward USDC or an actually owed asset
are allowed. Proceeds stay in the vault. Per-leg/slippage/aggregate loss and
turnover limits still apply; consumed window budget is not reset by cash flows.
The next 17,280-ledger window permits another bounded recovery budget. No new
borrowing can finance an exit.

[`local-smoke.py`](../scripts/local-smoke.py) constructs a complete swap/action
payload and invokes public unwind independently of target execution. Its router
is a fixture. A live quote must supply the exact pinned Soroswap distribution and
auth tree, with minimum output checked independently onchain. Do not substitute
a direct venue call. Resimulate after every successful leg, use the new nonce,
and retry the bounded USDC exit once liquidity is available.

`recover_supply(asset, amount)` withdraws unencumbered supply and verifies spot
plus claim conservation, with at most one atomic-unit rounding loss. `repay(asset,
amount)` verifies that debt reduction covers actual cash spent. Both are public,
price-independent and callable while paused. They cannot trade, release collateral
or increase debt. Material unclaimed rewards still require realization. Frozen
protocols or insufficient pool liquidity can block recovery; preserve shares and
retry after the external condition changes. `claim_rewards()` uses only the
configured pool/reward identity and verifies actual receipts.

For the separate borrowing fixture, repay liabilities and release collateral
under fresh pool health checks before standard payout. The current conservative
interface blocks USDC exits with any residual debt/collateral. This does not prove
D9 atomic/stressed settlement or authorize a flagship borrowing mode.

## Price outage and pause

A guardian pause prevents issuance and ordinary execution; eligible exits,
fee checkpoints and safe recovery remain callable. Resume is a timelocked admin
change with valid pricing. Oracle restoration alone does not restore pool liquidity.

Direct holders can call `redeem_in_kind(shares, owner, receiver, minima, deadline)`
when all holdings are transferable spot tokens and actual supply/collateral/debt
and material unclaimed rewards are zero. Read the canonical asset order from
`config` to construct `minima`. Catch up management fees first. With unavailable
NAV the event reports the waived share fraction and retained HWM, not a USDC fee.
The DeFindex adapter always requires USDC and cannot take this exception.

## TTL and restoration

Instance state contains config, supply, HWM/dust/time, nonce/version, guard,
target and pending timelock. Balances and allowances use persistent storage.
Mutations extend instance and touched entries at a 172,800-ledger threshold to
518,400 ledgers. `extend_ttl(owners, spenders)` touches up to 16 owners/spenders;
code TTL must also be maintained through Stellar's code-extension transaction.
Blend positions belong to the pool and require its own supported restoration.
Temporary storage is not used for ownership/accounting.

Before expiry, use `stellar contract extend` for code/instance and invoke
`extend_ttl` for relevant balances/allowances. After archival, restore the original
entries with the pinned CLI:

```sh
stellar contract restore --wasm-hash "$WASM_HASH" --source "$PAYER" --network "$NETWORK"
stellar contract restore --id "$VAULT" --source "$PAYER" --network "$NETWORK"
stellar contract restore --id "$VAULT" --key-xdr "$BALANCE_KEY_XDR" \
  --durability persistent --source "$PAYER" --network "$NETWORK"
```

Persistent keys are XDR ScVal vectors `[Symbol("Balance"), Address(owner)]` and
`[Symbol("Allowance"), Address(owner), Address(spender)]`. Encode the reviewed
ScVal with `stellar xdr encode --type ScVal`. Restore the adapter and upstream
entries as needed. These are network restore operations, never constructor calls
or data reinitialization. Original allowance expiration still applies afterward.
Compare supply, HWM, nonce/version, balances and pending change to the pre-archive
snapshot before resuming. The D2 test renews TTL on an unchanged ledger snapshot
and verifies these values; a live-network eviction/restore drill remains an
operational activation check, not claimed by that native test.
