use crate::types::{Config, Error, Pending, State, Target};
use soroban_sdk::{contracttype, panic_with_error, Address, Env};

pub const TTL_THRESHOLD: u32 = 172_800;
pub const TTL_EXTEND: u32 = 518_400;
#[contracttype]
#[derive(Clone)]
pub enum Key {
    Config,
    State,
    Guard,
    Balance(Address),
    Allowance(Address, Address),
    Target,
    Pending,
}
#[contracttype]
#[derive(Clone)]
pub struct Allowance {
    pub amount: i128,
    pub expiration: u32,
}

#[inline(never)]
pub fn config(e: &Env) -> Config {
    e.storage().instance().get(&Key::Config).unwrap()
}
#[inline(never)]
pub fn state(e: &Env) -> State {
    e.storage().instance().get(&Key::State).unwrap()
}
#[inline(never)]
pub fn save(e: &Env, s: &State) {
    e.storage().instance().set(&Key::State, s);
}
#[inline(never)]
pub fn target(e: &Env) -> Target {
    e.storage()
        .instance()
        .get(&Key::Target)
        .unwrap_or_else(|| panic_with_error!(e, Error::Target))
}
#[inline(never)]
pub fn pending(e: &Env) -> Pending {
    e.storage()
        .instance()
        .get(&Key::Pending)
        .unwrap_or_else(|| panic_with_error!(e, Error::Timelock))
}
#[inline(never)]
pub fn enter(e: &Env) {
    if e.storage().instance().get(&Key::Guard).unwrap_or(false) {
        panic_with_error!(e, Error::Reentrant);
    }
    e.storage().instance().set(&Key::Guard, &true);
    e.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
}
#[inline(never)]
pub fn leave(e: &Env) {
    e.storage().instance().set(&Key::Guard, &false);
}
#[inline(never)]
pub fn balance(e: &Env, who: &Address) -> i128 {
    e.storage()
        .persistent()
        .get(&Key::Balance(who.clone()))
        .unwrap_or(0)
}
#[inline(never)]
pub fn write_balance(e: &Env, who: &Address, amount: i128) {
    let key = Key::Balance(who.clone());
    e.storage().persistent().set(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
}
#[inline(never)]
pub fn allowance(e: &Env, from: &Address, spender: &Address) -> i128 {
    let value: Option<Allowance> = e
        .storage()
        .persistent()
        .get(&Key::Allowance(from.clone(), spender.clone()));
    value
        .filter(|a| a.expiration >= e.ledger().sequence())
        .map(|a| a.amount)
        .unwrap_or(0)
}
#[inline(never)]
pub fn spend(e: &Env, from: &Address, spender: &Address, amount: i128) {
    if from == spender {
        return;
    }
    let key = Key::Allowance(from.clone(), spender.clone());
    let mut a: Allowance = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, Error::Unauthorized));
    if a.expiration < e.ledger().sequence() || a.amount < amount {
        panic_with_error!(e, Error::Unauthorized);
    }
    a.amount -= amount;
    e.storage().persistent().set(&key, &a);
    e.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
}
