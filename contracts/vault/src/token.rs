use crate::{
    math::{add, nonnegative},
    storage,
    types::Error,
};
use soroban_sdk::{panic_with_error, Address, Env};
use soroban_token_sdk::TokenUtils;

#[inline(never)]
pub fn credit(e: &Env, who: &Address, amount: i128) {
    nonnegative(e, amount);
    storage::write_balance(e, who, add(e, storage::balance(e, who), amount));
    if amount > 0 {
        TokenUtils::new(e)
            .events()
            .mint(e.current_contract_address(), who.clone(), amount);
    }
}
#[inline(never)]
pub fn debit(e: &Env, who: &Address, amount: i128) {
    nonnegative(e, amount);
    if *who == e.current_contract_address() {
        panic_with_error!(e, Error::SeedLocked);
    }
    let old = storage::balance(e, who);
    if old < amount {
        panic_with_error!(e, Error::InsufficientBalance);
    }
    storage::write_balance(e, who, old - amount);
}
#[inline(never)]
pub fn transfer(e: &Env, from: &Address, to: &Address, amount: i128) {
    debit(e, from, amount);
    // Transfers to the vault are disallowed: only the funded seed is locked there.
    if *to == e.current_contract_address() {
        panic_with_error!(e, Error::SeedLocked);
    }
    storage::write_balance(e, to, add(e, storage::balance(e, to), amount));
    TokenUtils::new(e)
        .events()
        .transfer(from.clone(), to.clone(), amount);
}
#[inline(never)]
pub fn burn(e: &Env, from: &Address, amount: i128) {
    debit(e, from, amount);
    let mut s = storage::state(e);
    s.supply -= amount;
    storage::save(e, &s);
    TokenUtils::new(e).events().burn(from.clone(), amount);
}
