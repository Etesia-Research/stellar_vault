#![no_std]
//! Deliberately simple, admin-minted LOCAL TEST assets, marks and swap liquidity.
//! Never a production dependency or a D3/D4 protocol-compatibility claim.
#![allow(clippy::too_many_arguments)]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token::TokenClient, vec, Address, BytesN,
    Env, String, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Mark {
    pub price: i128,
    pub reference: i128,
    pub timestamp: u64,
    pub ready: bool,
}
#[contracttype]
#[derive(Clone)]
pub enum Protocol {
    Soroswap = 0,
    Phoenix = 1,
    Aqua = 2,
    Comet = 3,
}
#[contracttype]
#[derive(Clone)]
pub struct DexDistribution {
    pub protocol_id: Protocol,
    pub path: Vec<Address>,
    pub parts: u32,
    pub bytes: Option<Vec<BytesN<32>>>,
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
            reference: 1_000_000_000_000,
            timestamp: e.ledger().timestamp(),
            ready: true,
        }
    }
    pub fn swap_exact_tokens_for_tokens(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Vec<Vec<i128>> {
        to.require_auth();
        assert!(deadline >= e.ledger().timestamp());
        assert!(!distribution.is_empty());
        assert!(amount_in >= amount_out_min);
        TokenClient::new(&e, &token_in).transfer(&to, &e.current_contract_address(), &amount_in);
        TokenClient::new(&e, &token_out).transfer(&e.current_contract_address(), &to, &amount_in);
        vec![&e, vec![&e, amount_in, amount_in]]
    }
}
