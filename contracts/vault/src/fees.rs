use crate::{
    math::{add, fraction, mul_div, sub},
    storage, token,
    types::*,
};
use soroban_sdk::{panic_with_error, symbol_short, Env};

#[inline(never)]
pub fn validate(e: &Env, f: &Fees) {
    if f.management_bps > MAX_MANAGEMENT_BPS
        || f.performance_bps > MAX_PERFORMANCE_BPS
        || f.recipient == e.current_contract_address()
    {
        panic_with_error!(e, Error::Invalid);
    }
}

/// At most one year per checkpoint. Priced calls require a fully caught-up state.
#[inline(never)]
pub fn management(e: &Env, s: &mut State, f: &Fees, checkpoint: bool) -> i128 {
    let elapsed = e
        .ledger()
        .timestamp()
        .checked_sub(s.last_fee)
        .unwrap_or_else(|| panic_with_error!(e, Error::Invalid));
    if elapsed > YEAR && !checkpoint {
        panic_with_error!(e, Error::CatchUp);
    }
    let dt = elapsed.min(YEAR);
    let rate_time = i128::from(f.management_bps) * i128::from(dt);
    let denominator = BPS * i128::from(YEAR) - rate_time;
    let scaled = add(
        e,
        fraction(e, s.supply, rate_time, denominator, SCALE),
        s.management_dust,
    );
    let minted = add(
        e,
        mul_div(e, s.supply, rate_time, denominator, false),
        scaled / SCALE,
    );
    s.management_dust = scaled % SCALE;
    s.supply = add(e, s.supply, minted);
    s.last_fee += dt;
    minted
}

#[inline(never)]
pub fn performance(e: &Env, s: &mut State, f: &Fees, equity: i128) -> i128 {
    if equity <= 0 {
        return 0;
    }
    let baseline = mul_div(e, s.high_water, s.supply, SCALE, true);
    let gain = (equity - baseline).max(0);
    if gain == 0 {
        return 0;
    }
    let charge = add(
        e,
        mul_div(e, gain, i128::from(f.performance_bps) * SCALE, BPS, false),
        s.performance_dust,
    );
    if charge == 0 {
        s.high_water = s.high_water.max(mul_div(e, equity, SCALE, s.supply, true));
        return 0;
    }
    let scaled_equity = mul_div(e, equity, SCALE, 1, false);
    let minted = mul_div(e, charge, s.supply, sub(e, scaled_equity, charge), false);
    // Without a whole fee share, retain H; the uncrystallized gain still includes this entitlement.
    if minted == 0 {
        return 0;
    }
    s.supply = add(e, s.supply, minted);
    let paid = mul_div(e, minted, scaled_equity, s.supply, false);
    s.performance_dust = sub(e, charge, paid);
    s.high_water = s.high_water.max(mul_div(e, equity, SCALE, s.supply, true));
    minted
}

#[inline(never)]
pub fn preview(e: &Env, equity: i128) -> State {
    let mut s = storage::state(e);
    let f = storage::config(e).fees;
    management(e, &mut s, &f, false);
    performance(e, &mut s, &f, equity);
    s
}

#[inline(never)]
pub fn settle(e: &Env, equity: Option<i128>, checkpoint: bool) -> State {
    let mut s = storage::state(e);
    let f = storage::config(e).fees;
    let start = s.last_fee;
    let m = management(e, &mut s, &f, checkpoint);
    let p = equity.map(|a| performance(e, &mut s, &f, a)).unwrap_or(0);
    token::credit(e, &f.recipient, add(e, m, p));
    storage::save(e, &s);
    e.events().publish(
        (symbol_short!("fees"), 1u32),
        (m, p, start, s.last_fee, s.high_water, f),
    );
    s
}
