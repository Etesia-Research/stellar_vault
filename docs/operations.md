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
translated to Aggregator v3 `routes` of `Hop::Soroswap` through configured pairs;
the vault authorizes only the input transfer to v3. Read the current nonce first.
Only withdrawals, repayments, collateral release and swaps toward USDC or an
actually owed asset are allowed. Non-USDC debt-asset purchases cannot exceed debt minus existing spot
holdings, and must be consumed by debt repayment within the same unwind (one atom
of repayment rounding per asset is tolerated). Reward-only assets can be sold but
cannot be purchased. Proceeds stay in the vault. Per-leg/slippage/aggregate loss
and turnover limits still apply; consumed window budget is not reset by cash flows.
The next 17,280-ledger window permits another bounded recovery budget. No new
borrowing can finance an exit.

[`local-smoke.py`](../scripts/local-smoke.py) constructs a complete swap/action
payload and invokes public unwind independently of target execution, through
mainnet v3, router, factory and pair code on an Etesia-made
local pair. A live quote's route amounts must sum to `amount` minus v3's
`protocol_bps` fee, and `minimum` must clear the vault's Reflector floor, which is
checked onchain. Do not substitute a direct venue call. Resimulate after every
successful leg, use the new nonce, and retry the bounded USDC exit once liquidity
is available.

`recover_supply(asset, amount)` withdraws unencumbered supply and verifies spot
plus claim conservation, with at most one atomic-unit rounding loss. `repay(asset,
amount)` verifies that debt reduction covers actual cash spent. Both are public,
price-independent and callable while paused. They cannot trade, release collateral
or increase debt. Material unclaimed rewards still require realization. Frozen
protocols or insufficient pool liquidity can block recovery; preserve shares and
retry after the external condition changes. `claim_rewards()` uses only the
configured pool/reward identity and verifies actual receipts.

For the separate borrowing fixture, repay liabilities and release collateral
under fresh pool health checks before standard payout. An unwind may finish above
the debt ceiling or below the health buffer only when it strictly reduces debt
and debt/equity without worsening pool risk-weighted health. This permits partial
swap-and-repay recovery after adverse moves; collateral release that worsens an
already deficient health ratio remains rejected. The current conservative
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

### Local archival drill (D6-04, support only)

On 2026-11-11 an isolated protocol-30 node with `min_persistent_ttl` lowered to 16
ledgers archived the vault code entry (`ac188cbf…1b47`, the hash of both testnet
vaults in the pre-reset manifest) and a fixture token's instance, code and
persistent balance. The three `stellar contract restore` forms above restored them,
and the balance read back unchanged. Since protocol 23 a simulated invocation also
lists archived footprint entries for automatic restore, and the sending transaction
restores them itself; the drill observed this as well. RPC 30 reports an archived
entry with `liveUntilLedgerSeq` 0. The vault's own instance and balance keys were not
archived, because every touch extends them to 518,400 ledgers. The transcript is in
the workspace record `doc/evidence/d6-04-archival-restore-local.md`. This is a
fixture-network check; the live-network drill above remains open.

## Deployment

Code citations in this and the next two sections are at vault `11992f9`.

The constructor `__constructor(config, seed_from)` requires the admin's
authorization, and the seed payer's when it differs
(`contracts/vault/src/lib.rs:118-122`). It rejects invalid fee terms
(`fees.rs:9-17`) and out-of-range limits: at most 8 assets in strict address order,
16 route contracts and 7 pool assets; `max_price_age` 1–3,600 s; slippage at most
100 bps, leg 2,000, turnover 10,000, loss 200, debt 5,000; health at least 12,500
bps; a nonzero cooldown (`lib.rs:124-164`). Each asset's decimals are read from its
token contract, and exactly one Settlement asset must be the 7-decimal USDC. The
constructor transfers the 1,000 USDC seed (`types.rs:7`) from `seed_from` and
credits the seed shares to the vault itself (`lib.rs:165-195`); deposits to the
vault's own address fail `SeedLocked` (`lib.rs:66-67`).

The contract has no upgrade entry point. Code, assets, dependency identities, risk
limits and borrowing mode cannot change after deployment; only fee terms, roles and
`Resume` change, through the timelock below ([interfaces](interfaces.md)). A new
code revision or limit set is a new deployment, and holders leave the old one
through its own exits. Deploy the optimized WASM built by `scripts/check.sh`
([local RPC guide](local-rpc.md)), record its SHA-256 and the network's current
resource limits, and verify `config()` against the reviewed manifest before any
deposit. The testnet vaults were deployed by `scripts/testnet-deploy.py vaults` into
`deploy/testnet/pre-2026-12-16-reset.json`; a post-reset redeploy has not been run.

## Signer roles

`Config` holds three role addresses: `admin`, `executor` and `guardian`
(`types.rs:78-80`). Fee shares go to `fees.recipient` (`types.rs:63`,
`fees.rs:96`), which signs nothing in the vault.

| Role | Can call | Code |
| --- | --- | --- |
| Admin | Authorize the constructor; `announce(change)`, one pending change at a time | `lib.rs:119`; `governance.rs:21-26` |
| Executor | `publish_target` and `execute`, both refused while paused | `execution.rs:526-531, 545-556` |
| Guardian | `pause()`; `cancel(hash)` of the pending change | `governance.rs:11-13, 60-65` |
| New role holder | Authorizes its own appointment inside `apply_change` | `governance.rs:85-89` |
| Anyone | `apply_change(hash)` at or after `earliest`; `unwind`, `recover_supply`, `repay`, `claim_rewards`; `collect_fees`, `checkpoint_fees`, `extend_ttl` | `governance.rs:71-75`; `execution.rs:606, 650, 669, 683`; `lib.rs:470-481, 543` |
| Holder or operator | Deposits, withdrawals and redemptions for its own shares | `lib.rs:69-71, 90` |

The guardian cannot unpause: `Resume` is an admin announcement that applies after
the delay and only with positive equity (`governance.rs:96-101`). Pausing blocks
issuance, `publish_target` and `execute`, not exits or recovery
(`lib.rs:49-52`, `execution.rs:529, 554`). The executor chooses targets within the
hard limits; the contract does not prove that it ran the quant model
([interfaces](interfaces.md)).

## Role rotation and key management

A role changes only through the timelock. `announce(Change::Role(role, address))`
stores one pending change whose hash binds the network id, the vault, the current
`version` and the change (`governance.rs:30-43`). It applies no earlier than
`ROLE_DELAY` = 34,560 ledgers later, about 48 hours at 5 s per ledger
(`governance.rs:44-48`, `types.rs:12`). `apply_change` checks the hash and the
ledger, requires the new address's authorization, refuses the vault's own address,
sets the role and increments `version` (`governance.rs:71-115`). A version bump
makes any other announcement for the old version unappliable. While a change is
pending, `announce` fails `Timelock` (#18), so a pending `Resume` blocks a rotation
until it is applied or cancelled.

Rotation steps:

1. Read `config()`, `state()` and `pending_change()`; the last must be empty.
2. Prepare the new account and confirm it can sign (see custody below).
3. Admin: announce, and record the returned hash and `earliest` from the
   `announce` event:

```sh
stellar contract invoke --id "$VAULT" --source "$ADMIN_KEY" --network "$NETWORK" -- \
  announce --change '{"Role":["Executor","'"$NEW_ADDRESS"'"]}'
```

4. During the delay, the guardian checks the pending change against the reviewed
   request and cancels anything else with `cancel --hash "$HASH"`.
5. At or after `earliest`, apply with the new holder's authorization, for example
   with the new holder as the source account:

```sh
stellar contract invoke --id "$VAULT" --source "$NEW_KEY" --network "$NETWORK" -- \
  apply_change --hash "$HASH"
```

6. Read `config()` and `state()` back: the new address, `version` + 1, no pending
   change. Archive the receipts.
7. Retire the old key. The old address loses vault authority at the apply ledger.
   For the executor, switch the keeper's sealed `ETESIA_EXECUTOR_SECRET` in the same
   window; until then its `execute` fails authorization. Then move the old
   account's remaining XLM.

A simulation of steps 3 and 5 on the testnet guard vault on 2026-11-11 parsed both
argument forms and failed `Timelock` (#18), as expected with `Resume` pending.
No role has been rotated on testnet.

**Compromise.** A stolen admin key can only announce, so the guardian's `cancel`
within the delay is the control. A stolen guardian key can pause and cancel every
announcement, including the guardian's own replacement, so it can hold the vault
paused indefinitely; exits and recovery stay available, and the remedy is a new
deployment. A stolen executor key is bounded by the onchain limits; pause the vault,
then rotate the executor. The D6-03 monitor does not alert on `pending_change()`,
so the guardian's review in step 4 is manual.

**Custody.** Each role is a Stellar account address. In Soroban authorization a
Stellar account uses its classic signers and its medium threshold
([Stellar authorization](https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization#stellar-account)),
so a role can be an m-of-n account configured with `set-options` signers, weights
and thresholds ([multisig](https://developers.stellar.org/docs/learn/fundamentals/transactions/signatures-multisig#thresholds)).
A master weight of 0 with too few other signers locks the account. The testnet roles
are single keys: on 2026-11-11 Horizon showed each role account with only its master
key at weight 1 and thresholds 0/0/0. The admin `GAQKH3PL…DUVY` serves both vaults;
the flagship executor `GDGCL46K…JT7N` and guardian `GCVI25VJ…FLPQ`, the guard
executor `GDTIM3OL…3PR6` and guardian `GCHQQLF3…DTEH` are separate. Local keys live
in the gitignored CLI config `.stellar/`; the keeper's executor secret is a sealed
Railway variable. Multisig custody and an executor key held apart from the public
API process are mainnet readiness items, not testnet facts.
