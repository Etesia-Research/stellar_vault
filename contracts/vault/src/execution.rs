// The pinned Soroswap ABI has eight parameters.
#![allow(clippy::too_many_arguments)]
use crate::{
    blend::{self, PoolClient, Request},
    fees,
    math::{add, mul_div, positive},
    pricing,
    storage::{self, Key},
    types::*,
    Vault, VaultArgs, VaultClient,
};
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contractclient, contractimpl, panic_with_error, symbol_short, vec,
    xdr::ToXdr,
    Address, BytesN, Env, IntoVal, Map, Symbol, Vec,
};

#[allow(dead_code)]
#[contractclient(name = "RouterClient")]
pub trait Router {
    fn swap_exact_tokens_for_tokens(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Vec<Vec<i128>>;
}
#[inline(never)]
fn hash_target(e: &Env, target: &Target) -> BytesN<32> {
    e.crypto()
        .sha256(&(symbol_short!("target_v1"), target.clone()).to_xdr(e))
        .to_bytes()
}
#[inline(never)]
fn holdings_hash(e: &Env) -> BytesN<32> {
    let c = storage::config(e);
    let mut spot = Vec::new(e);
    for asset in c.assets.iter() {
        spot.push_back((asset.address.clone(), pricing::spot(e, &asset.address)));
    }
    let positions = c
        .pool
        .map(|p| PoolClient::new(e, &p).get_positions(&e.current_contract_address()));
    e.crypto().sha256(&(spot, positions).to_xdr(e)).to_bytes()
}

#[inline(never)]
fn mark_value(e: &Env, address: &Address, amount: i128, ceil: bool) -> i128 {
    if amount == 0 {
        return 0;
    }
    let a = pricing::asset(e, address);
    let p = pricing::price(e, &a).unwrap_or_else(|err| panic_with_error!(e, err));
    pricing::value(e, amount, &a, p, ceil)
}
#[inline(never)]
fn validate_target(e: &Env, t: &Target) {
    let c = storage::config(e);
    if t.vault != e.current_contract_address()
        || t.network != e.ledger().network_id()
        || t.expiry < e.ledger().timestamp()
        || t.expiry > e.ledger().timestamp() + 86_400
        || t.assets.len() != c.assets.len()
        || t.weights.len() != c.assets.len()
        || t.yield_bps > 1_000
        || t.equity <= 0
        || mul_div(
            e,
            (t.equity - pricing::positive_equity(e)).abs(),
            BPS,
            t.equity,
            true,
        ) > 100
        || t.supply <= 0
        || mul_div(
            e,
            (t.supply - fees::preview(e, t.equity).supply).abs(),
            BPS,
            t.supply,
            true,
        ) > 100
        || t.holdings != holdings_hash(e)
    {
        panic_with_error!(e, Error::Target);
    }
    let mut sum = 0u32;
    let mut reserve = 0u32;
    let mut settlement = 0u32;
    for (i, a) in c.assets.iter().enumerate() {
        let weight = t.weights.get(i as u32).unwrap();
        if t.assets.get(i as u32).unwrap() != a.address || weight > 10_000 {
            panic_with_error!(e, Error::Target);
        }
        sum = sum
            .checked_add(weight)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        match a.kind {
            AssetKind::Settlement => {
                if weight < 250 {
                    panic_with_error!(e, Error::Target);
                }
                settlement = weight;
            }
            AssetKind::Xlm => {
                if !(250..=4_000).contains(&weight) {
                    panic_with_error!(e, Error::Target);
                }
            }
            AssetKind::Risk => {
                if weight > 4_000 {
                    panic_with_error!(e, Error::Target);
                }
            }
            AssetKind::Reserve => reserve += weight,
            AssetKind::Reward => {
                if weight != 0 {
                    panic_with_error!(e, Error::Target);
                }
            }
        }
    }
    if sum != 10_000 || settlement - 250 != (reserve + settlement - 250) * t.yield_bps / 10_000 {
        panic_with_error!(e, Error::Target);
    }
    if t.yield_bps > 0 && (c.pool.is_none() || !c.pool_assets.contains(c.usdc)) {
        panic_with_error!(e, Error::Target);
    }
}
#[inline(never)]
fn yield_limit(e: &Env, t: &Target, equity: i128) -> i128 {
    let c = storage::config(e);
    let mut trend_bps = 0;
    for (i, a) in c.assets.iter().enumerate() {
        let w = t.weights.get(i as u32).unwrap();
        match a.kind {
            AssetKind::Risk => trend_bps += w,
            AssetKind::Xlm => trend_bps += w - 250,
            _ => (),
        }
    }
    mul_div(
        e,
        equity,
        i128::from(9_500 - trend_bps) * i128::from(t.yield_bps),
        BPS * BPS,
        false,
    )
}
#[inline(never)]
fn distance(e: &Env, t: &Target, equity: i128, holdings: &Vec<Holding>) -> i128 {
    let mut sum = 0;
    for (i, h) in holdings.iter().enumerate() {
        let amount = add(e, add(e, h.spot, h.supply), h.collateral);
        let net = mark_value(e, &h.asset, amount, false) - mark_value(e, &h.asset, h.debt, true);
        let desired = mul_div(
            e,
            equity,
            i128::from(t.weights.get(i as u32).unwrap()),
            BPS,
            false,
        );
        sum = add(e, sum, (net - desired).abs());
        if h.asset == storage::config(e).usdc {
            sum = add(e, sum, (h.supply - yield_limit(e, t, equity)).abs());
        }
    }
    sum
}

// Only nested swap frames at pinned route contracts and bounded path-token transfers
// can be authorized. This data grants authorization, never arbitrary invocation.
#[inline(never)]
fn check_auth(
    e: &Env,
    swap: &Swap,
    entries: &Vec<RouteAuth>,
    depth: u32,
    count: &mut u32,
    intermediate: &mut Map<Address, i128>,
) -> (i128, Vec<InvokerContractAuthEntry>) {
    let c = storage::config(e);
    if depth > 5 || entries.len() > 24 {
        panic_with_error!(e, Error::Limit);
    }
    let mut spent = 0;
    let mut authorized = Vec::new(e);
    for entry in entries.iter() {
        *count += 1;
        if *count > 48 {
            panic_with_error!(e, Error::Limit);
        }
        let invocation = match entry {
            RouteAuth::Transfer(token, to, amount) => {
                positive(e, amount);
                let mut text = [0u8; 56];
                to.to_string().copy_into_slice(&mut text);
                let is_input = token == swap.token_in;
                if (!is_input && !intermediate.contains_key(token.clone()))
                    || text[0] != b'C'
                    || to == c.executor
                    || to == c.admin
                    || to == c.guardian
                    || to == c.fees.recipient
                    || to == e.current_contract_address()
                {
                    panic_with_error!(e, Error::Unauthorized);
                }
                if is_input {
                    spent = add(e, spent, amount);
                } else {
                    let total = add(e, intermediate.get(token.clone()).unwrap(), amount);
                    if mark_value(e, &token, total, true)
                        > mul_div(
                            e,
                            mark_value(e, &swap.token_in, swap.amount, true),
                            BPS + i128::from(c.max_slippage_bps),
                            BPS,
                            false,
                        )
                    {
                        panic_with_error!(e, Error::Limit);
                    }
                    intermediate.set(token.clone(), total);
                }
                SubContractInvocation {
                    context: ContractContext {
                        contract: token,
                        fn_name: symbol_short!("transfer"),
                        args: (e.current_contract_address(), to, amount).into_val(e),
                    },
                    sub_invocations: vec![e],
                }
            }
            RouteAuth::Invoke(frame) => {
                if !c.route_contracts.contains(frame.contract.clone())
                    || !(frame.function == Symbol::new(e, "swap_exact_tokens_for_tokens")
                        || frame.function == symbol_short!("swap")
                        || frame.function == Symbol::new(e, "swap_chained")
                        || frame.function == Symbol::new(e, "swap_exact_amount_in"))
                {
                    panic_with_error!(e, Error::Unauthorized);
                }
                let (amount, children) =
                    check_auth(e, swap, &frame.children, depth + 1, count, intermediate);
                spent = add(e, spent, amount);
                SubContractInvocation {
                    context: ContractContext {
                        contract: frame.contract,
                        fn_name: frame.function,
                        args: frame.args,
                    },
                    sub_invocations: children,
                }
            }
        };
        authorized.push_back(InvokerContractAuthEntry::Contract(invocation));
    }
    (spent, authorized)
}

#[inline(never)]
fn swap(e: &Env, s: &Swap, deadline: u64) -> i128 {
    let c = storage::config(e);
    positive(e, s.amount);
    positive(e, s.minimum);
    if s.token_in == s.token_out || s.distribution.is_empty() || s.distribution.len() > 15 {
        panic_with_error!(e, Error::Invalid);
    }
    pricing::asset(e, &s.token_in);
    pricing::asset(e, &s.token_out);
    let mut parts = 0u32;
    let mut intermediate = Map::new(e);
    for route in s.distribution.iter() {
        if route.parts == 0
            || route.path.len() < 2
            || route.path.len() > 5
            || route.path.first().unwrap() != s.token_in
            || route.path.last().unwrap() != s.token_out
        {
            panic_with_error!(e, Error::Invalid);
        }
        let mut seen = Vec::new(e);
        for a in route.path.iter() {
            pricing::asset(e, &a);
            if seen.contains(a.clone()) {
                panic_with_error!(e, Error::Invalid);
            }
            seen.push_back(a.clone());
            if a != s.token_in && a != s.token_out {
                intermediate.set(a, 0i128);
            }
        }
        parts = parts
            .checked_add(route.parts)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        if route.protocol_id == Protocol::Aqua
            && route.bytes.as_ref().map(|v| v.len()) != Some(route.path.len() - 1)
        {
            panic_with_error!(e, Error::Invalid);
        }
    }
    let value = mark_value(e, &s.token_in, s.amount, true);
    if mark_value(e, &s.token_out, s.minimum, false)
        < mul_div(e, value, BPS - i128::from(c.max_slippage_bps), BPS, true)
    {
        panic_with_error!(e, Error::Slippage);
    }
    let (authorized_amount, authorization) =
        check_auth(e, s, &s.auth, 0, &mut 0, &mut intermediate);
    if authorized_amount != s.amount {
        panic_with_error!(e, Error::Unauthorized);
    }
    let mut intermediate_before = Map::new(e);
    for (asset, _) in intermediate.iter() {
        intermediate_before.set(asset.clone(), pricing::spot(e, &asset));
    }
    let before_in = pricing::spot(e, &s.token_in);
    let before_out = pricing::spot(e, &s.token_out);
    e.authorize_as_current_contract(authorization);
    RouterClient::new(e, &c.router).swap_exact_tokens_for_tokens(
        &s.token_in,
        &s.token_out,
        &s.amount,
        &s.minimum,
        &s.distribution,
        &e.current_contract_address(),
        &deadline,
    );
    for (asset, before) in intermediate_before.iter() {
        if pricing::spot(e, &asset) != before {
            panic_with_error!(e, Error::Slippage);
        }
    }
    let received = pricing::spot(e, &s.token_out) - before_out;
    if before_in - pricing::spot(e, &s.token_in) != s.amount || received < s.minimum {
        panic_with_error!(e, Error::Slippage);
    }
    value
}
#[inline(never)]
fn pool_action(e: &Env, kind: u32, asset: &Address, amount: i128) {
    let c = storage::config(e);
    positive(e, amount);
    if !c.pool_assets.contains(asset.clone()) {
        panic_with_error!(e, Error::Invalid);
    }
    if !c.borrowing && (2..=4).contains(&kind) {
        panic_with_error!(e, Error::BorrowingDisabled);
    }
    if kind == 0 && *asset != c.usdc {
        panic_with_error!(e, Error::Invalid);
    }
    let pool = c
        .pool
        .unwrap_or_else(|| panic_with_error!(e, Error::Invalid));
    let this = e.current_contract_address();
    let before = pricing::spot(e, asset);
    let before_positions = PoolClient::new(e, &pool).get_positions(&this);
    let reserve = PoolClient::new(e, &pool).get_reserve(asset);
    let mut expected = before_positions.clone();
    let map = match kind {
        0 | 1 => &mut expected.supply,
        2 | 3 => &mut expected.collateral,
        _ => &mut expected.liabilities,
    };
    let index = reserve.config.index;
    let old = map.get(index).unwrap_or(0);
    let rate = if kind >= 4 {
        reserve.data.d_rate
    } else {
        reserve.data.b_rate
    };
    // Bound the request itself: Blend transfers the requested repayment before refunding excess.
    let amount = if kind == 5 {
        amount.min(mul_div(e, old, rate, PRICE_SCALE, true))
    } else {
        amount
    };
    positive(e, amount);
    let tokens = mul_div(e, amount, PRICE_SCALE, rate, matches!(kind, 1 | 3 | 4));
    let reducing = matches!(kind, 1 | 3 | 5);
    let next = if reducing {
        old - tokens.min(old)
    } else {
        add(e, old, tokens)
    };
    if next == old {
        panic_with_error!(e, Error::Invalid);
    }
    if next == 0 {
        map.remove(index);
    } else {
        map.set(index, next);
    }
    let transferred = if matches!(kind, 1 | 3) && tokens > old {
        mul_div(e, old, rate, PRICE_SCALE, false)
    } else {
        amount
    };
    if kind == 0 || kind == 2 || kind == 5 {
        let transfer = amount;
        e.authorize_as_current_contract(vec![
            e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: asset.clone(),
                    fn_name: symbol_short!("transfer"),
                    args: (&this, &pool, transfer).into_val(e),
                },
                sub_invocations: vec![e],
            }),
        ]);
    }
    let result = PoolClient::new(e, &pool).submit(
        &this,
        &this,
        &this,
        &vec![
            e,
            Request {
                request_type: kind,
                address: asset.clone(),
                amount,
            },
        ],
    );
    let after = pricing::spot(e, asset);
    let delta = if matches!(kind, 0 | 2 | 5) {
        before - after
    } else {
        after - before
    };
    if delta != transferred
        || result != expected
        || PoolClient::new(e, &pool).get_positions(&this) != expected
    {
        panic_with_error!(e, Error::Invalid);
    }
    e.events().publish(
        (symbol_short!("position"), 1u32),
        (kind, asset.clone(), amount, before_positions, result),
    );
}
#[inline(never)]
fn limits(e: &Env, before: i128, after: i128, turnover: i128) {
    let c = storage::config(e);
    let mut s = storage::state(e);
    if e.ledger().sequence() - s.window_ledger >= 17_280 {
        s.window_ledger = e.ledger().sequence();
        s.window_equity = before;
        s.turnover = 0;
        s.loss = 0;
    }
    s.turnover = add(e, s.turnover, turnover);
    s.loss = add(e, s.loss, (before - after).max(0));
    let base = s.window_equity.min(before);
    if s.turnover > mul_div(e, base, i128::from(c.max_turnover_bps), BPS, false)
        || s.loss > mul_div(e, base, i128::from(c.max_loss_bps), BPS, false)
    {
        panic_with_error!(e, Error::Limit);
    }
    s.last_execution = e.ledger().sequence();
    s.nonce = s
        .nonce
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
    storage::save(e, &s);
}
#[inline(never)]
fn run(e: &Env, actions: &Vec<Action>, deadline: u64, recovery: bool, before: i128) -> i128 {
    let c = storage::config(e);
    if actions.is_empty() || actions.len() > 8 {
        panic_with_error!(e, Error::Limit);
    }
    let mut turnover = 0;
    for action in actions.iter() {
        let value = match action {
            Action::Swap(s) => {
                if recovery && s.token_out != c.usdc {
                    let debt = pricing::holdings(e, true)
                        .iter()
                        .find(|h| h.asset == s.token_out)
                        .map(|h| h.debt)
                        .unwrap_or(0);
                    if debt == 0 {
                        panic_with_error!(e, Error::Invalid);
                    }
                }
                swap(e, &s, deadline)
            }
            other => {
                let (kind, asset, amount) = match other {
                    Action::Supply(a, n) => (0, a, n),
                    Action::Withdraw(a, n) => (1, a, n),
                    Action::Collateral(a, n) => (2, a, n),
                    Action::Release(a, n) => (3, a, n),
                    Action::Borrow(a, n) => (4, a, n),
                    Action::Repay(a, n) => (5, a, n),
                    _ => unreachable!(),
                };
                if recovery && (kind == 0 || kind == 2 || kind == 4) {
                    panic_with_error!(e, Error::Paused);
                }
                let value = mark_value(e, &asset, amount, true);
                pool_action(e, kind, &asset, amount);
                value
            }
        };
        if value > mul_div(e, before, i128::from(c.max_leg_bps), BPS, false) {
            panic_with_error!(e, Error::Limit);
        }
        turnover = add(e, turnover, value);
    }
    turnover
}
#[inline(never)]
fn check_positions(e: &Env, equity: i128, t: Option<&Target>, holdings: &Vec<Holding>) {
    let c = storage::config(e);
    let mut debt = 0;
    for h in holdings.iter() {
        let a = pricing::asset(e, &h.asset);
        let gross = mark_value(
            e,
            &h.asset,
            add(e, add(e, h.spot, h.supply), h.collateral),
            false,
        );
        debt = add(e, debt, mark_value(e, &h.asset, h.debt, true));
        if t.is_some()
            && !c.borrowing
            && matches!(a.kind, AssetKind::Risk | AssetKind::Xlm)
            && gross > mul_div(e, equity, 4_000, BPS, false)
        {
            panic_with_error!(e, Error::Limit);
        }
        if let Some(t) = t {
            if h.supply > 0 && (h.asset != c.usdc || h.supply > yield_limit(e, t, equity)) {
                panic_with_error!(e, Error::Limit);
            }
        }
    }
    if debt > mul_div(e, equity, i128::from(c.max_debt_bps), BPS, false) {
        panic_with_error!(e, Error::Limit);
    }
    blend::check_health(e);
}

#[contractimpl]
impl Vault {
    pub fn holdings_hash(e: Env) -> BytesN<32> {
        holdings_hash(&e)
    }
    pub fn target_hash(e: Env, target: Target) -> BytesN<32> {
        hash_target(&e, &target)
    }
    pub fn target(e: Env) -> Target {
        storage::target(&e)
    }
    pub fn publish_target(e: Env, target: Target) -> BytesN<32> {
        storage::enter(&e);
        storage::config(&e).executor.require_auth();
        if storage::state(&e).paused {
            panic_with_error!(&e, Error::Paused);
        }
        validate_target(&e, &target);
        let old: Option<Target> = e.storage().instance().get(&Key::Target);
        if old.map(|t| t.epoch >= target.epoch).unwrap_or(false) {
            panic_with_error!(&e, Error::Replay);
        }
        let hash = hash_target(&e, &target);
        e.storage().instance().set(&Key::Target, &target);
        e.events()
            .publish((symbol_short!("target"), 1u32), (hash.clone(), target));
        storage::leave(&e);
        hash
    }
    pub fn execute(e: Env, plan: Plan) {
        storage::enter(&e);
        let c = storage::config(&e);
        c.executor.require_auth();
        let old = pricing::holdings(&e, true);
        let before = pricing::equity_of(&e, &old).unwrap_or_else(|err| panic_with_error!(&e, err));
        positive(&e, before);
        let s = fees::settle(&e, Some(before), false);
        let target = storage::target(&e);
        if s.paused {
            panic_with_error!(&e, Error::Paused);
        }
        if plan.nonce != s.nonce || plan.version != s.version {
            panic_with_error!(&e, Error::Replay);
        }
        if plan.deadline < e.ledger().timestamp() || target.expiry < e.ledger().timestamp() {
            panic_with_error!(&e, Error::Expired);
        }
        if plan.epoch != target.epoch
            || plan.target != hash_target(&e, &target)
            || mul_div(
                &e,
                (target.supply - s.supply).abs(),
                BPS,
                target.supply,
                true,
            ) > 100
            || mul_div(&e, (before - target.equity).abs(), BPS, target.equity, true) > 100
        {
            panic_with_error!(&e, Error::Target);
        }
        if e.ledger().sequence() < s.last_execution + c.cooldown {
            panic_with_error!(&e, Error::Limit);
        }
        let old_distance = distance(&e, &target, before, &old);
        let turnover = run(&e, &plan.actions, plan.deadline, false, before);
        let new = pricing::holdings(&e, true);
        let after = pricing::equity_of(&e, &new).unwrap_or_else(|err| panic_with_error!(&e, err));
        positive(&e, after);
        if distance(&e, &target, after, &new) >= old_distance {
            panic_with_error!(&e, Error::Target);
        }
        for (i, h) in new.iter().enumerate() {
            let a = c.assets.get(i as u32).unwrap();
            if matches!(a.kind, AssetKind::Settlement | AssetKind::Xlm) {
                let old_spot = mark_value(&e, &h.asset, old.get(i as u32).unwrap().spot, false);
                let new_spot = mark_value(&e, &h.asset, h.spot, false);
                if new_spot < old_spot.min(mul_div(&e, after, 250, BPS, false)) {
                    panic_with_error!(&e, Error::Limit);
                }
            }
        }
        check_positions(&e, after, Some(&target), &new);
        limits(&e, before, after, turnover);
        e.events().publish(
            (symbol_short!("plan"), 1u32),
            (plan.nonce, plan.target, before, after, turnover),
        );
        storage::leave(&e);
    }
    pub fn unwind(e: Env, actions: Vec<Action>, nonce: u64, deadline: u64) {
        storage::enter(&e);
        if storage::state(&e).nonce != nonce {
            panic_with_error!(&e, Error::Replay);
        }
        if deadline < e.ledger().timestamp() {
            panic_with_error!(&e, Error::Expired);
        }
        let old = pricing::holdings(&e, true);
        let before = pricing::equity_of(&e, &old).unwrap_or_else(|err| panic_with_error!(&e, err));
        positive(&e, before);
        fees::settle(&e, Some(before), false);
        let cash = pricing::spot(&e, &storage::config(&e).usdc);
        let turnover = run(&e, &actions, deadline, true, before);
        let new = pricing::holdings(&e, true);
        let after = pricing::equity_of(&e, &new).unwrap_or_else(|err| panic_with_error!(&e, err));
        positive(&e, after);
        let mut reduced = pricing::spot(&e, &storage::config(&e).usdc) > cash;
        for (i, h) in new.iter().enumerate() {
            let previous = old.get(i as u32).unwrap();
            if h.debt > previous.debt
                || h.collateral > previous.collateral
                || h.supply > previous.supply
            {
                panic_with_error!(&e, Error::Limit);
            }
            reduced |= h.debt < previous.debt || h.supply < previous.supply;
        }
        if !reduced {
            panic_with_error!(&e, Error::Limit);
        }
        check_positions(&e, after, None, &new);
        limits(&e, before, after, turnover);
        e.events()
            .publish((symbol_short!("unwind"), 1u32), (nonce, before, after));
        storage::leave(&e);
    }
    pub fn recover_supply(e: Env, asset: Address, amount: i128) {
        storage::enter(&e);
        // Exact underlying recovery needs no market price and grants no trading authority.
        let before = pricing::holdings(&e, true);
        pool_action(&e, 1, &asset, amount);
        let after = pricing::holdings(&e, true);
        let i = before.iter().position(|h| h.asset == asset).unwrap() as u32;
        let b = before.get(i).unwrap();
        let a = after.get(i).unwrap();
        if a.spot <= b.spot
            || a.supply >= b.supply
            || a.spot + a.supply + 1 < b.spot + b.supply
            || a.debt != b.debt
            || a.collateral != b.collateral
        {
            panic_with_error!(&e, Error::Limit);
        }
        storage::leave(&e);
    }
    pub fn repay(e: Env, asset: Address, amount: i128) {
        storage::enter(&e);
        let before = pricing::holdings(&e, true);
        pool_action(&e, 5, &asset, amount);
        let after = pricing::holdings(&e, true);
        let i = before.iter().position(|h| h.asset == asset).unwrap() as u32;
        let b = before.get(i).unwrap();
        let a = after.get(i).unwrap();
        if a.debt >= b.debt || b.spot - a.spot > b.debt - a.debt + 1 || a.collateral != b.collateral
        {
            panic_with_error!(&e, Error::Limit);
        }
        storage::leave(&e);
    }
    pub fn claim_rewards(e: Env) -> i128 {
        storage::enter(&e);
        let c = storage::config(&e);
        let reward = c
            .assets
            .iter()
            .find(|a| a.kind == AssetKind::Reward)
            .unwrap_or_else(|| panic_with_error!(&e, Error::Rewards));
        let pool = PoolClient::new(
            &e,
            &c.pool
                .unwrap_or_else(|| panic_with_error!(&e, Error::Invalid)),
        );
        let before = pricing::spot(&e, &reward.address);
        let mut ids = Vec::new(&e);
        for a in c.pool_assets.iter() {
            let i = pool.get_reserve(&a).config.index;
            ids.push_back(i * 2);
            ids.push_back(i * 2 + 1);
        }
        let amount = pool.claim(
            &e.current_contract_address(),
            &ids,
            &e.current_contract_address(),
        );
        if amount < 0 || pricing::spot(&e, &reward.address) - before != amount {
            panic_with_error!(&e, Error::Rewards);
        }
        e.events()
            .publish((symbol_short!("rewards"), 1u32), (reward.address, amount));
        storage::leave(&e);
        amount
    }
}
