#![no_std]

pub mod blend;
mod execution;
mod fees;
mod governance;
mod math;
pub mod pricing;
mod storage;
mod token;
pub mod types;

use math::{add, mul_div, nonnegative, positive};
use soroban_sdk::{
    contract, contractimpl, map, panic_with_error, symbol_short, token::TokenClient, Address, Env,
    String, Vec,
};
use soroban_token_sdk::{metadata::TokenMetadata, TokenUtils};
use storage::Key;
use types::*;

#[contract]
pub struct Vault;

fn flow_event(
    e: &Env,
    deposit: bool,
    operator: Address,
    from: Address,
    to: Address,
    assets: i128,
    shares: i128,
) {
    let data = map![
        e,
        (symbol_short!("assets"), assets),
        (symbol_short!("shares"), shares)
    ];
    if deposit {
        e.events()
            .publish((symbol_short!("deposit"), operator, from, to), data);
    } else {
        e.events()
            .publish((symbol_short!("withdraw"), operator, to, from), data);
    }
}
fn enter_flow(e: &Env, issue: bool) -> (i128, State) {
    storage::enter(e);
    if issue && storage::state(e).paused {
        panic_with_error!(e, Error::Paused);
    }
    let a = pricing::positive_equity(e);
    let s = fees::settle(e, Some(a), false);
    (a, s)
}
fn check_deadline(e: &Env, deadline: u64) {
    if deadline < e.ledger().timestamp() {
        panic_with_error!(e, Error::Expired);
    }
}
fn issue(e: &Env, assets: i128, shares: i128, receiver: Address, from: Address, operator: Address) {
    positive(e, assets);
    positive(e, shares);
    if receiver == e.current_contract_address() {
        panic_with_error!(e, Error::SeedLocked);
    }
    operator.require_auth();
    if from != operator {
        from.require_auth();
    }
    let c = storage::config(e);
    let before = pricing::spot(e, &c.usdc);
    TokenClient::new(e, &c.usdc).transfer(&from, &e.current_contract_address(), &assets);
    if pricing::spot(e, &c.usdc) - before != assets {
        panic_with_error!(e, Error::Invalid);
    }
    let mut s = storage::state(e);
    s.supply = add(e, s.supply, shares);
    s.window_equity = add(e, s.window_equity, assets);
    storage::save(e, &s);
    token::credit(e, &receiver, shares);
    flow_event(e, true, operator, from, receiver, assets, shares);
    storage::leave(e);
}
fn exit(e: &Env, assets: i128, shares: i128, receiver: Address, owner: Address, operator: Address) {
    positive(e, assets);
    positive(e, shares);
    operator.require_auth();
    storage::spend(e, &owner, &operator, shares);
    let c = storage::config(e);
    if receiver == e.current_contract_address() {
        panic_with_error!(e, Error::Invalid);
    }
    if pricing::spot(e, &c.usdc) < assets {
        panic_with_error!(e, Error::Liquidity);
    }
    // Leveraged exits require the explicit exposure unwind before paying USDC.
    if pricing::holdings(e, true)
        .iter()
        .any(|h| h.debt != 0 || h.collateral != 0)
    {
        panic_with_error!(e, Error::ExternalClaim);
    }
    let mut s = storage::state(e);
    s.performance_dust = mul_div(e, s.performance_dust, s.supply - shares, s.supply, false);
    s.window_equity = (s.window_equity - assets).max(0);
    storage::save(e, &s);
    token::burn(e, &owner, shares);
    TokenClient::new(e, &c.usdc).transfer(&e.current_contract_address(), &receiver, &assets);
    flow_event(e, false, operator, owner, receiver, assets, shares);
    storage::leave(e);
}

#[contractimpl]
impl Vault {
    pub fn __constructor(e: Env, config: Config, seed_from: Address) {
        config.admin.require_auth();
        if seed_from != config.admin {
            seed_from.require_auth();
        }
        fees::validate(&e, &config.fees);
        if config.assets.is_empty()
            || config.assets.len() > 8
            || config.route_contracts.len() > 16
            || config.pool_assets.len() > 7
            || config.max_price_age == 0
            || config.max_price_age > 3600
            || config.max_divergence_bps > 500
            || config.max_slippage_bps > 100
            || config.max_leg_bps > 2_000
            || config.max_turnover_bps > 10_000
            || config.max_loss_bps > 200
            || config.max_debt_bps > 5_000
            || config.min_health_bps < 12_500
            || config.cooldown == 0
        {
            panic_with_error!(&e, Error::Invalid);
        }
        let mut previous: Option<Address> = None;
        let mut usdc = false;
        for a in config.assets.iter() {
            if a.decimals > 12 || previous.as_ref().map(|p| *p >= a.address).unwrap_or(false) {
                panic_with_error!(&e, Error::Invalid);
            }
            if TokenClient::new(&e, &a.address).decimals() != a.decimals {
                panic_with_error!(&e, Error::Invalid);
            }
            if a.kind == AssetKind::Settlement {
                if a.address != config.usdc || a.decimals != 7 {
                    panic_with_error!(&e, Error::Invalid);
                }
                usdc = true;
            }
            previous = Some(a.address);
        }
        if !usdc || (config.pool.is_none() && !config.pool_assets.is_empty()) {
            panic_with_error!(&e, Error::Invalid);
        }
        for a in config.pool_assets.iter() {
            if !config.assets.iter().any(|x| x.address == a) {
                panic_with_error!(&e, Error::Invalid);
            }
        }
        e.storage().instance().set(&Key::Config, &config);
        storage::enter(&e);
        let before = pricing::spot(&e, &config.usdc);
        TokenClient::new(&e, &config.usdc).transfer(
            &seed_from,
            &e.current_contract_address(),
            &SEED_ASSETS,
        );
        if pricing::spot(&e, &config.usdc) - before != SEED_ASSETS {
            panic_with_error!(&e, Error::Invalid);
        }
        let seed = SEED_ASSETS * SHARE_MULTIPLIER;
        storage::save(
            &e,
            &State {
                supply: seed,
                high_water: SCALE / SHARE_MULTIPLIER,
                last_fee: e.ledger().timestamp(),
                management_dust: 0,
                performance_dust: 0,
                nonce: 0,
                version: 1,
                paused: false,
                last_execution: 0,
                window_ledger: e.ledger().sequence(),
                window_equity: SEED_ASSETS,
                turnover: 0,
                loss: 0,
            },
        );
        token::credit(&e, &e.current_contract_address(), seed);
        TokenUtils::new(&e).metadata().set_metadata(&TokenMetadata {
            decimal: 12,
            name: String::from_str(&e, "Etesia Vault"),
            symbol: String::from_str(&e, "ETESIA"),
        });
        e.events()
            .publish((symbol_short!("init"), 1u32), (config, SEED_ASSETS, seed));
        storage::leave(&e);
    }
    pub fn config(e: Env) -> Config {
        storage::config(&e)
    }
    pub fn state(e: Env) -> State {
        storage::state(&e)
    }
    pub fn holdings(e: Env) -> Vec<Holding> {
        pricing::holdings(&e, true)
    }
    pub fn query_asset(e: Env) -> Address {
        storage::config(&e).usdc
    }
    pub fn total_supply(e: Env) -> i128 {
        storage::state(&e).supply
    }
    pub fn total_assets(e: Env) -> i128 {
        pricing::equity(&e)
    }
    pub fn balance(e: Env, id: Address) -> i128 {
        storage::balance(&e, &id)
    }
    pub fn decimals(_e: Env) -> u32 {
        12
    }
    pub fn name(e: Env) -> String {
        TokenUtils::new(&e).metadata().get_metadata().name
    }
    pub fn symbol(e: Env) -> String {
        TokenUtils::new(&e).metadata().get_metadata().symbol
    }
    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        storage::allowance(&e, &from, &spender)
    }
    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        storage::enter(&e);
        from.require_auth();
        nonnegative(&e, amount);
        if from == e.current_contract_address()
            || (amount > 0 && expiration_ledger < e.ledger().sequence())
        {
            panic_with_error!(&e, Error::Invalid);
        }
        let key = Key::Allowance(from.clone(), spender.clone());
        e.storage().persistent().set(
            &key,
            &storage::Allowance {
                amount,
                expiration: expiration_ledger,
            },
        );
        e.storage()
            .persistent()
            .extend_ttl(&key, storage::TTL_THRESHOLD, storage::TTL_EXTEND);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
        storage::leave(&e);
    }
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        storage::enter(&e);
        from.require_auth();
        token::transfer(&e, &from, &to, amount);
        storage::leave(&e);
    }
    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        storage::enter(&e);
        spender.require_auth();
        nonnegative(&e, amount);
        storage::spend(&e, &from, &spender, amount);
        token::transfer(&e, &from, &to, amount);
        storage::leave(&e);
    }
    pub fn burn(e: Env, from: Address, amount: i128) {
        enter_flow(&e, false);
        from.require_auth();
        token::burn(&e, &from, amount);
        storage::leave(&e);
    }
    pub fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        enter_flow(&e, false);
        spender.require_auth();
        nonnegative(&e, amount);
        storage::spend(&e, &from, &spender, amount);
        token::burn(&e, &from, amount);
        storage::leave(&e);
    }
    pub fn convert_to_shares(e: Env, assets: i128) -> i128 {
        nonnegative(&e, assets);
        let a = pricing::positive_equity(&e);
        mul_div(&e, assets, fees::preview(&e, a).supply, a, false)
    }
    pub fn convert_to_assets(e: Env, shares: i128) -> i128 {
        nonnegative(&e, shares);
        let a = pricing::positive_equity(&e);
        mul_div(&e, shares, a, fees::preview(&e, a).supply, false)
    }
    pub fn preview_deposit(e: Env, assets: i128) -> i128 {
        Self::convert_to_shares(e, assets)
    }
    pub fn preview_redeem(e: Env, shares: i128) -> i128 {
        Self::convert_to_assets(e, shares)
    }
    pub fn preview_mint(e: Env, shares: i128) -> i128 {
        nonnegative(&e, shares);
        let a = pricing::positive_equity(&e);
        mul_div(&e, shares, a, fees::preview(&e, a).supply, true)
    }
    pub fn preview_withdraw(e: Env, assets: i128) -> i128 {
        nonnegative(&e, assets);
        let a = pricing::positive_equity(&e);
        mul_div(&e, assets, fees::preview(&e, a).supply, a, true)
    }
    pub fn max_deposit(e: Env, receiver: Address) -> i128 {
        if storage::state(&e).paused
            || receiver == e.current_contract_address()
            || pricing::equity_result(&e).map(|a| a <= 0).unwrap_or(true)
        {
            0
        } else {
            MAX_AMOUNT / SHARE_MULTIPLIER
        }
    }
    pub fn max_mint(e: Env, receiver: Address) -> i128 {
        let max = Self::max_deposit(e.clone(), receiver);
        if max == 0 {
            0
        } else {
            Self::preview_deposit(e, max)
        }
    }
    pub fn liquid_assets(e: Env) -> i128 {
        pricing::spot(&e, &storage::config(&e).usdc)
    }
    pub fn max_withdraw(e: Env, owner: Address) -> i128 {
        if owner == e.current_contract_address()
            || pricing::equity_result(&e).map(|a| a <= 0).unwrap_or(true)
            || pricing::holdings(&e, true)
                .iter()
                .any(|h| h.debt > 0 || h.collateral > 0)
        {
            return 0;
        }
        Self::convert_to_assets(e.clone(), storage::balance(&e, &owner)).min(Self::liquid_assets(e))
    }
    pub fn max_redeem(e: Env, owner: Address) -> i128 {
        let max = Self::max_withdraw(e.clone(), owner.clone());
        if max == 0 {
            0
        } else {
            Self::convert_to_shares(e.clone(), max).min(storage::balance(&e, &owner))
        }
    }
    pub fn deposit(
        e: Env,
        assets: i128,
        receiver: Address,
        from: Address,
        operator: Address,
    ) -> i128 {
        Self::deposit_bounded(e, assets, receiver, from, operator, 1, u64::MAX)
    }
    pub fn deposit_bounded(
        e: Env,
        assets: i128,
        receiver: Address,
        from: Address,
        operator: Address,
        min_shares: i128,
        deadline: u64,
    ) -> i128 {
        check_deadline(&e, deadline);
        positive(&e, assets);
        positive(&e, min_shares);
        let (a, s) = enter_flow(&e, true);
        let shares = mul_div(&e, assets, s.supply, a, false);
        if shares < min_shares {
            panic_with_error!(&e, Error::Slippage);
        }
        issue(&e, assets, shares, receiver, from, operator);
        shares
    }
    pub fn mint(e: Env, shares: i128, receiver: Address, from: Address, operator: Address) -> i128 {
        Self::mint_bounded(e, shares, receiver, from, operator, MAX_AMOUNT, u64::MAX)
    }
    pub fn mint_bounded(
        e: Env,
        shares: i128,
        receiver: Address,
        from: Address,
        operator: Address,
        max_assets: i128,
        deadline: u64,
    ) -> i128 {
        check_deadline(&e, deadline);
        positive(&e, shares);
        nonnegative(&e, max_assets);
        let (a, s) = enter_flow(&e, true);
        let assets = mul_div(&e, shares, a, s.supply, true);
        if assets > max_assets {
            panic_with_error!(&e, Error::Slippage);
        }
        issue(&e, assets, shares, receiver, from, operator);
        assets
    }
    pub fn withdraw(
        e: Env,
        assets: i128,
        receiver: Address,
        owner: Address,
        operator: Address,
    ) -> i128 {
        Self::withdraw_bounded(e, assets, receiver, owner, operator, MAX_AMOUNT, u64::MAX)
    }
    pub fn withdraw_bounded(
        e: Env,
        assets: i128,
        receiver: Address,
        owner: Address,
        operator: Address,
        max_shares: i128,
        deadline: u64,
    ) -> i128 {
        check_deadline(&e, deadline);
        positive(&e, assets);
        nonnegative(&e, max_shares);
        let (a, s) = enter_flow(&e, false);
        let shares = mul_div(&e, assets, s.supply, a, true);
        if shares > max_shares {
            panic_with_error!(&e, Error::Slippage);
        }
        exit(&e, assets, shares, receiver, owner, operator);
        shares
    }
    pub fn redeem(
        e: Env,
        shares: i128,
        receiver: Address,
        owner: Address,
        operator: Address,
    ) -> i128 {
        Self::redeem_bounded(e, shares, receiver, owner, operator, 1, u64::MAX)
    }
    pub fn redeem_bounded(
        e: Env,
        shares: i128,
        receiver: Address,
        owner: Address,
        operator: Address,
        min_assets: i128,
        deadline: u64,
    ) -> i128 {
        check_deadline(&e, deadline);
        positive(&e, shares);
        nonnegative(&e, min_assets);
        let (a, s) = enter_flow(&e, false);
        let assets = mul_div(&e, shares, a, s.supply, false);
        if assets < min_assets {
            panic_with_error!(&e, Error::Slippage);
        }
        exit(&e, assets, shares, receiver, owner, operator);
        assets
    }
    pub fn collect_fees(e: Env) -> State {
        storage::enter(&e);
        let a = pricing::equity(&e);
        let s = fees::settle(&e, Some(a), false);
        storage::leave(&e);
        s
    }
    pub fn checkpoint_fees(e: Env) -> State {
        storage::enter(&e);
        let s = fees::settle(&e, None, true);
        storage::leave(&e);
        s
    }
    pub fn redeem_in_kind(
        e: Env,
        shares: i128,
        owner: Address,
        receiver: Address,
        minima: Vec<i128>,
        deadline: u64,
    ) -> Vec<i128> {
        storage::enter(&e);
        owner.require_auth();
        check_deadline(&e, deadline);
        positive(&e, shares);
        if receiver == e.current_contract_address() {
            panic_with_error!(&e, Error::Invalid);
        }
        let holdings = pricing::transferable(&e);
        if holdings.len() != minima.len() {
            panic_with_error!(&e, Error::Invalid);
        }
        let equity = pricing::equity_result(&e).ok();
        let mut s = fees::settle(&e, equity, false);
        if shares > storage::balance(&e, &owner) {
            panic_with_error!(&e, Error::InsufficientBalance);
        }
        s.performance_dust = mul_div(&e, s.performance_dust, s.supply - shares, s.supply, false);
        s.window_equity = mul_div(&e, s.window_equity, s.supply - shares, s.supply, false);
        storage::save(&e, &s);
        token::burn(&e, &owner, shares);
        let mut paid = Vec::new(&e);
        for (i, h) in holdings.iter().enumerate() {
            let amount = mul_div(&e, h.spot, shares, s.supply, false);
            let minimum = minima.get(i as u32).unwrap();
            nonnegative(&e, minimum);
            if amount < minimum {
                panic_with_error!(&e, Error::Slippage);
            }
            if amount > 0 {
                TokenClient::new(&e, &h.asset).transfer(
                    &e.current_contract_address(),
                    &receiver,
                    &amount,
                );
            }
            paid.push_back(amount);
        }
        if equity.is_none() {
            e.events().publish(
                (symbol_short!("waiver"), 1u32),
                (owner.clone(), shares, s.supply, s.high_water),
            );
        }
        e.events().publish(
            (symbol_short!("basket"), 1u32),
            (owner, receiver, shares, paid.clone()),
        );
        storage::leave(&e);
        paid
    }
    pub fn extend_ttl(e: Env, owners: Vec<Address>, spenders: Vec<Address>) {
        if owners.len() > 16 || spenders.len() > 16 {
            panic_with_error!(&e, Error::Limit);
        }
        storage::enter(&e);
        for owner in owners.iter() {
            let k = Key::Balance(owner.clone());
            if e.storage().persistent().has(&k) {
                e.storage().persistent().extend_ttl(
                    &k,
                    storage::TTL_THRESHOLD,
                    storage::TTL_EXTEND,
                );
            }
            for spender in spenders.iter() {
                let k = Key::Allowance(owner.clone(), spender);
                if e.storage().persistent().has(&k) {
                    e.storage().persistent().extend_ttl(
                        &k,
                        storage::TTL_THRESHOLD,
                        storage::TTL_EXTEND,
                    );
                }
            }
        }
        storage::leave(&e);
    }
}

#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "testutils"))]
pub mod testutils;
