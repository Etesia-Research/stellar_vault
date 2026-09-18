use crate::{
    blend::{PoolClient, Reserve},
    math::{add, mul_div, sub},
    storage,
    types::*,
};
use soroban_sdk::{contractclient, panic_with_error, token::TokenClient, Address, Env, Vec};

#[contractclient(name = "OracleClient")]
pub trait Oracle {
    fn mark(e: Env, asset: Address) -> Mark;
}

#[inline(never)]
pub fn asset(e: &Env, address: &Address) -> Asset {
    storage::config(e)
        .assets
        .iter()
        .find(|a| a.address == *address)
        .unwrap_or_else(|| panic_with_error!(e, Error::Invalid))
}
#[inline(never)]
pub fn price(e: &Env, a: &Asset) -> Result<i128, Error> {
    let c = storage::config(e);
    if a.address == c.usdc {
        return Ok(PRICE_SCALE);
    }
    let mark = match OracleClient::new(e, &c.oracle).try_mark(&a.address) {
        Ok(Ok(m)) => m,
        _ => return Err(Error::Pricing),
    };
    let now = e.ledger().timestamp();
    if !mark.ready
        || mark.price <= 0
        || mark.reference <= 0
        || mark.timestamp > now
        || now - mark.timestamp > c.max_price_age
    {
        return Err(Error::Pricing);
    }
    if mul_div(
        e,
        (mark.price - mark.reference).abs(),
        BPS,
        mark.reference,
        true,
    ) > i128::from(c.max_divergence_bps)
    {
        return Err(Error::Pricing);
    }
    Ok(mark.price)
}
#[inline(never)]
pub fn value(e: &Env, amount: i128, a: &Asset, p: i128, ceil: bool) -> i128 {
    mul_div(
        e,
        amount,
        p,
        10i128.pow(a.decimals) * (PRICE_SCALE / 10_000_000),
        ceil,
    )
}
#[inline(never)]
pub fn spot(e: &Env, a: &Address) -> i128 {
    TokenClient::new(e, a).balance(&e.current_contract_address())
}

#[inline(never)]
fn pending_reward(
    e: &Env,
    pool: &PoolClient,
    r: &Reserve,
    id: u32,
    balance: i128,
    total: i128,
) -> i128 {
    let Some(mut emission) = pool.get_reserve_emissions(&id) else {
        return 0;
    };
    let time = e.ledger().timestamp().min(emission.expiration);
    if time > emission.last_time && total > 0 {
        emission.index = add(
            e,
            emission.index,
            mul_div(
                e,
                i128::from(time - emission.last_time) * i128::from(emission.eps),
                r.scalar,
                total,
                false,
            ),
        );
    }
    let user = pool.get_user_emissions(&e.current_contract_address(), &id);
    let index = user.as_ref().map(|u| u.index).unwrap_or(0);
    let accrued = user.map(|u| u.accrued).unwrap_or(0);
    add(
        e,
        accrued,
        mul_div(
            e,
            balance,
            sub(e, emission.index, index),
            r.scalar * 10_000_000,
            false,
        ),
    )
}

/// A single authoritative position read. Blend get_reserve accrues to this ledger.
#[inline(never)]
pub fn holdings(e: &Env, require_rewards_realized: bool) -> Vec<Holding> {
    let c = storage::config(e);
    let mut result = Vec::new(e);
    for a in c.assets.iter() {
        result.push_back(Holding {
            asset: a.address.clone(),
            spot: spot(e, &a.address),
            supply: 0,
            collateral: 0,
            debt: 0,
        });
    }
    if let Some(pool_address) = c.pool {
        let pool = PoolClient::new(e, &pool_address);
        let positions = pool.get_positions(&e.current_contract_address());
        let mut known = Vec::new(e);
        for address in c.pool_assets.iter() {
            let r = pool.get_reserve(&address);
            let a = asset(e, &address);
            if r.asset != address
                || r.config.decimals != a.decimals
                || r.scalar != 10i128.pow(a.decimals)
                || r.data.last_time != e.ledger().timestamp()
            {
                panic_with_error!(e, Error::Invalid);
            }
            let i = r.config.index;
            known.push_back(i);
            let b = positions.supply.get(i).unwrap_or(0);
            let collateral = positions.collateral.get(i).unwrap_or(0);
            let d = positions.liabilities.get(i).unwrap_or(0);
            if require_rewards_realized
                && (pending_reward(
                    e,
                    &pool,
                    &r,
                    i * 2 + 1,
                    add(e, b, collateral),
                    r.data.b_supply,
                ) > 0
                    || pending_reward(e, &pool, &r, i * 2, d, r.data.d_supply) > 0)
            {
                panic_with_error!(e, Error::Rewards);
            }
            let idx = c.assets.iter().position(|a| a.address == address).unwrap() as u32;
            let mut h = result.get(idx).unwrap();
            h.supply = mul_div(e, b, r.data.b_rate, PRICE_SCALE, false);
            h.collateral = mul_div(e, collateral, r.data.b_rate, PRICE_SCALE, false);
            h.debt = mul_div(e, d, r.data.d_rate, PRICE_SCALE, true);
            result.set(idx, h);
        }
        for map in [
            positions.supply,
            positions.collateral,
            positions.liabilities,
        ] {
            for (index, amount) in map.iter() {
                if amount < 0 || !known.contains(index) {
                    panic_with_error!(e, Error::ExternalClaim);
                }
            }
        }
    }
    result
}

#[inline(never)]
pub fn equity_result(e: &Env) -> Result<i128, Error> {
    equity_of(e, &holdings(e, true))
}

pub fn equity_of(e: &Env, holdings: &Vec<Holding>) -> Result<i128, Error> {
    let mut total = 0;
    for h in holdings.iter() {
        if h.spot == 0 && h.supply == 0 && h.collateral == 0 && h.debt == 0 {
            continue;
        }
        let a = asset(e, &h.asset);
        let p = price(e, &a)?;
        total = add(
            e,
            total,
            value(
                e,
                add(e, add(e, h.spot, h.supply), h.collateral),
                &a,
                p,
                false,
            ),
        );
        total = sub(e, total, value(e, h.debt, &a, p, true));
    }
    Ok(total)
}
#[inline(never)]
pub fn equity(e: &Env) -> i128 {
    equity_result(e).unwrap_or_else(|err| panic_with_error!(e, err))
}
#[inline(never)]
pub fn positive_equity(e: &Env) -> i128 {
    let a = equity(e);
    if a <= 0 {
        panic_with_error!(e, Error::Insolvent);
    }
    a
}
#[inline(never)]
pub fn transferable(e: &Env) -> Vec<Holding> {
    let h = holdings(e, true);
    if h.iter()
        .any(|h| h.supply != 0 || h.collateral != 0 || h.debt != 0)
    {
        panic_with_error!(e, Error::ExternalClaim);
    }
    h
}
