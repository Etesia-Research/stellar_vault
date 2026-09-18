// Wire types from Blend v2 ba22b487; see docs/interfaces.md.
use soroban_sdk::{contractclient, contracttype, Address, Env, Map, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolConfig {
    pub oracle: Address,      // the contract address of the oracle
    pub min_collateral: i128, // the minimum amount of collateral required to open a liability position
    pub bstop_rate: u32, // the rate the backstop takes on accrued debt interest, expressed in 7 decimals
    pub status: u32,     // the status of the pool
    pub max_positions: u32, // the maximum number of effective positions a single user can hold, and the max assets an auction can contain
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ReserveConfig {
    pub index: u32,       // the index of the reserve in the list
    pub decimals: u32,    // the decimals used in both the bToken and underlying contract
    pub c_factor: u32,    // the collateral factor for the reserve scaled expressed in 7 decimals
    pub l_factor: u32,    // the liability factor for the reserve scaled expressed in 7 decimals
    pub util: u32,        // the target utilization rate scaled expressed in 7 decimals
    pub max_util: u32,    // the maximum allowed utilization rate scaled expressed in 7 decimals
    pub r_base: u32, // the R0 value (base rate) in the interest rate formula scaled expressed in 7 decimals
    pub r_one: u32,  // the R1 value in the interest rate formula scaled expressed in 7 decimals
    pub r_two: u32,  // the R2 value in the interest rate formula scaled expressed in 7 decimals
    pub r_three: u32, // the R3 value in the interest rate formula scaled expressed in 7 decimals
    pub reactivity: u32, // the reactivity constant for the reserve scaled expressed in 7 decimals
    pub supply_cap: i128, // the total amount of underlying tokens that can be supplied to the reserve
    pub enabled: bool,    // the enabled flag of the reserve
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ReserveData {
    pub d_rate: i128,   // the conversion rate from dToken to underlying with 12 decimals
    pub b_rate: i128,   // the conversion rate from bToken to underlying with 12 decimals
    pub ir_mod: i128,   // the interest rate curve modifier with 7 decimals
    pub b_supply: i128, // the total supply of b tokens, in the underlying token's decimals
    pub d_supply: i128, // the total supply of d tokens, in the underlying token's decimals
    pub backstop_credit: i128, // the amount of underlying tokens currently owed to the backstop
    pub last_time: u64, // the last block the data was updated
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ReserveEmissionData {
    pub expiration: u64,
    pub eps: u64,
    pub index: i128,
    pub last_time: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UserEmissionData {
    pub index: i128,
    pub accrued: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Reserve {
    pub asset: Address,
    pub config: ReserveConfig,
    pub data: ReserveData,
    pub scalar: i128,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Positions {
    pub liabilities: Map<u32, i128>,
    pub collateral: Map<u32, i128>,
    pub supply: Map<u32, i128>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct Request {
    pub request_type: u32,
    pub address: Address,
    pub amount: i128,
}
#[contractclient(name = "PoolClient")]
pub trait Pool {
    fn get_config(e: Env) -> PoolConfig;
    fn get_reserve_list(e: Env) -> Vec<Address>;
    fn get_reserve(e: Env, asset: Address) -> Reserve;
    fn get_positions(e: Env, address: Address) -> Positions;
    fn submit(
        e: Env,
        from: Address,
        spender: Address,
        to: Address,
        requests: Vec<Request>,
    ) -> Positions;
    fn claim(e: Env, from: Address, reserve_token_ids: Vec<u32>, to: Address) -> i128;
    fn get_reserve_emissions(e: Env, reserve_token_id: u32) -> Option<ReserveEmissionData>;
    fn get_user_emissions(e: Env, user: Address, reserve_token_id: u32)
        -> Option<UserEmissionData>;
}

#[contracttype]
#[derive(Clone)]
pub enum OracleAsset {
    Stellar(Address),
    Other(soroban_sdk::Symbol),
}
#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}
#[contractclient(name = "HealthOracleClient")]
pub trait HealthOracle {
    fn lastprice(e: Env, asset: OracleAsset) -> Option<PriceData>;
}

pub fn health(e: &Env) -> (i128, i128) {
    use crate::{
        math::{add, mul_div},
        storage,
        types::Error,
    };
    use soroban_sdk::panic_with_error;
    let c = storage::config(e);
    let Some(address) = c.pool else {
        return (0, 0);
    };
    let pool = PoolClient::new(e, &address);
    let positions = pool.get_positions(&e.current_contract_address());
    if positions.liabilities.is_empty() {
        return (0, 0);
    }
    if !c.borrowing {
        panic_with_error!(e, Error::BorrowingDisabled);
    }
    let oracle = HealthOracleClient::new(e, &pool.get_config().oracle);
    let mut collateral = 0;
    let mut debt = 0;
    for a in c.pool_assets.iter() {
        let r = pool.get_reserve(&a);
        let b = positions.collateral.get(r.config.index).unwrap_or(0);
        let d = positions.liabilities.get(r.config.index).unwrap_or(0);
        if b == 0 && d == 0 {
            continue;
        }
        let p = oracle
            .lastprice(&OracleAsset::Stellar(a))
            .unwrap_or_else(|| panic_with_error!(e, Error::Pricing));
        let now = e.ledger().timestamp();
        if p.price <= 0 || p.timestamp > now || now - p.timestamp > c.max_price_age.min(86_400) {
            panic_with_error!(e, Error::Pricing);
        }
        let b_amount = mul_div(e, b, r.data.b_rate, 1_000_000_000_000, false);
        let d_amount = mul_div(e, d, r.data.d_rate, 1_000_000_000_000, true);
        let effective_b = mul_div(
            e,
            b_amount,
            i128::from(r.config.c_factor),
            10_000_000,
            false,
        );
        let effective_d = mul_div(e, d_amount, 10_000_000, i128::from(r.config.l_factor), true);
        collateral = add(
            e,
            collateral,
            mul_div(e, effective_b, p.price, r.scalar, false),
        );
        debt = add(e, debt, mul_div(e, effective_d, p.price, r.scalar, true));
    }
    (collateral, debt)
}

pub fn check_health(e: &Env) {
    let (collateral, debt) = health(e);
    if debt > 0
        && crate::math::mul_div(e, collateral, 10_000, debt, false)
            < i128::from(crate::storage::config(e).min_health_bps)
    {
        soroban_sdk::panic_with_error!(e, crate::types::Error::Limit);
    }
}
