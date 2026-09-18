#![no_std]
//! LOCAL ONLY: controlled Reflector ABI fixture, never a live price source.
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};
#[contracttype]
#[derive(Clone)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}
#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}
#[contract]
pub struct Fixture;
#[contractimpl]
impl Fixture {
    pub fn __constructor(e: Env, admin: Address, usdc: Address) {
        e.storage().instance().set(&symbol_short!("admin"), &admin);
        e.storage().instance().set(&symbol_short!("usdc"), &usdc);
    }
    pub fn base(e: Env) -> Asset {
        Asset::Stellar(e.storage().instance().get(&symbol_short!("usdc")).unwrap())
    }
    pub fn decimals(_e: Env) -> u32 {
        14
    }
    pub fn resolution(_e: Env) -> u32 {
        300
    }
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        e.storage().persistent().get(&asset)
    }
    pub fn update(e: Env, asset: Asset, price: i128, timestamp: u64) {
        let admin: Address = e.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        e.storage()
            .persistent()
            .set(&asset, &PriceData { price, timestamp });
    }
}
