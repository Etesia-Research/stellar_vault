#![no_std]
//! Deliberately simple, admin-minted LOCAL TEST assets and marks.
//! Never a production dependency or a D3/D4 protocol-compatibility claim.
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct Mark {
    pub price: i128,
    pub timestamp: u64,
}
#[contract]
pub struct Fixture;
#[contractimpl]
impl Fixture {
    pub fn __constructor(e: Env, admin: Address) {
        e.storage().instance().set(&symbol_short!("admin"), &admin);
    }
    pub fn decimals(_e: Env) -> u32 {
        7
    }
    pub fn symbol(e: Env) -> String {
        String::from_str(&e, "LOCAL")
    }
    pub fn name(e: Env) -> String {
        String::from_str(&e, "Etesia local fixture")
    }
    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage().persistent().get(&id).unwrap_or(0)
    }
    pub fn mint(e: Env, to: Address, amount: i128) {
        let admin: Address = e.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        assert!(amount > 0);
        let n = Self::balance(e.clone(), to.clone())
            .checked_add(amount)
            .unwrap();
        e.storage().persistent().set(&to, &n);
    }
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount >= 0);
        let balance = Self::balance(e.clone(), from.clone());
        assert!(balance >= amount);
        e.storage().persistent().set(&from, &(balance - amount));
        let n = Self::balance(e.clone(), to.clone())
            .checked_add(amount)
            .unwrap();
        e.storage().persistent().set(&to, &n);
    }
    pub fn mark(e: Env, asset: Address) -> Mark {
        let _ = asset;
        Mark {
            price: 1_000_000_000_000,
            timestamp: e.ledger().timestamp(),
        }
    }
}
