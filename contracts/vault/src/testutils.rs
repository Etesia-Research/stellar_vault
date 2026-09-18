// The pinned Soroswap ABI has eight parameters.
#![allow(clippy::too_many_arguments)]
extern crate std;
use crate::{blend::*, types::*, Vault, VaultClient};
use soroban_sdk::{
    contract, contractimpl, map, symbol_short,
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Map, Vec,
};

#[contract]
pub struct OracleFixture;
#[contractimpl]
impl OracleFixture {
    pub fn mark(e: Env, asset: Address) -> Mark {
        e.storage().instance().get(&asset).unwrap_or(Mark {
            price: PRICE_SCALE,
            reference: PRICE_SCALE,
            timestamp: e.ledger().timestamp(),
            ready: true,
        })
    }
    pub fn lastprice(e: Env, asset: OracleAsset) -> Option<PriceData> {
        let OracleAsset::Stellar(a) = asset else {
            return None;
        };
        let m = Self::mark(e, a);
        Some(PriceData {
            price: m.price,
            timestamp: m.timestamp,
        })
    }
}
#[contract]
pub struct PoolFixture;
#[contractimpl]
impl PoolFixture {
    pub fn __constructor(e: Env, assets: Vec<Address>, oracle: Address) {
        e.storage()
            .instance()
            .set(&symbol_short!("assets"), &assets);
        e.storage()
            .instance()
            .set(&symbol_short!("oracle"), &oracle);
    }
    pub fn get_config(e: Env) -> PoolConfig {
        PoolConfig {
            oracle: e
                .storage()
                .instance()
                .get(&symbol_short!("oracle"))
                .unwrap(),
            min_collateral: 1,
            bstop_rate: 0,
            status: 0,
            max_positions: 7,
        }
    }
    pub fn get_reserve_list(e: Env) -> Vec<Address> {
        e.storage()
            .instance()
            .get(&symbol_short!("assets"))
            .unwrap()
    }
    pub fn get_reserve(e: Env, asset: Address) -> Reserve {
        let assets = Self::get_reserve_list(e.clone());
        let index = assets.first_index_of(asset.clone()).unwrap();
        let b_rate = e
            .storage()
            .instance()
            .get(&(symbol_short!("brate"), asset.clone()))
            .unwrap_or(PRICE_SCALE);
        let d_rate = e
            .storage()
            .instance()
            .get(&(symbol_short!("drate"), asset.clone()))
            .unwrap_or(PRICE_SCALE);
        Reserve {
            asset,
            scalar: 10_000_000,
            config: ReserveConfig {
                index,
                decimals: 7,
                c_factor: 8_000_000,
                l_factor: 8_000_000,
                util: 7_500_000,
                max_util: 9_500_000,
                r_base: 0,
                r_one: 0,
                r_two: 0,
                r_three: 0,
                reactivity: 0,
                supply_cap: MAX_AMOUNT,
                enabled: true,
            },
            data: ReserveData {
                b_rate,
                d_rate,
                ir_mod: 10_000_000,
                b_supply: 100_000_000_000,
                d_supply: 50_000_000_000,
                backstop_credit: 0,
                last_time: e.ledger().timestamp(),
            },
        }
    }
    pub fn get_positions(e: Env, address: Address) -> Positions {
        e.storage().persistent().get(&address).unwrap_or(Positions {
            supply: Map::new(&e),
            collateral: Map::new(&e),
            liabilities: Map::new(&e),
        })
    }
    pub fn get_reserve_emissions(e: Env, reserve_token_id: u32) -> Option<ReserveEmissionData> {
        e.storage()
            .instance()
            .get(&(symbol_short!("emission"), reserve_token_id))
    }
    pub fn get_user_emissions(
        e: Env,
        user: Address,
        reserve_token_id: u32,
    ) -> Option<UserEmissionData> {
        e.storage().persistent().get(&(user, reserve_token_id))
    }
    pub fn claim(e: Env, from: Address, reserve_token_ids: Vec<u32>, to: Address) -> i128 {
        from.require_auth();
        let _ = reserve_token_ids;
        let amount: i128 = e
            .storage()
            .instance()
            .get(&symbol_short!("claim"))
            .unwrap_or(0);
        if amount > 0 {
            let reward: Address = e
                .storage()
                .instance()
                .get(&symbol_short!("reward"))
                .unwrap();
            TokenClient::new(&e, &reward).transfer(&e.current_contract_address(), &to, &amount);
            e.storage().instance().set(&symbol_short!("claim"), &0i128);
        }
        amount
            + e.storage()
                .instance()
                .get::<_, i128>(&symbol_short!("badclaim"))
                .unwrap_or(0)
    }
    pub fn submit(
        e: Env,
        from: Address,
        spender: Address,
        to: Address,
        requests: Vec<Request>,
    ) -> Positions {
        spender.require_auth();
        if from != spender {
            from.require_auth();
        }
        assert!(!e
            .storage()
            .instance()
            .get(&symbol_short!("frozen"))
            .unwrap_or(false));
        let mut p = Self::get_positions(e.clone(), from.clone());
        for r in requests.iter() {
            let reserve = Self::get_reserve(e.clone(), r.address.clone());
            let i = reserve.config.index;
            let token = TokenClient::new(&e, &r.address);
            let (map, rate) = match r.request_type {
                0 | 1 => (&mut p.supply, reserve.data.b_rate),
                2 | 3 => (&mut p.collateral, reserve.data.b_rate),
                _ => (&mut p.liabilities, reserve.data.d_rate),
            };
            let old = map.get(i).unwrap_or(0);
            let mut amount = r.amount;
            let shares = match r.request_type {
                0 | 2 => crate::math::mul_div(&e, amount, PRICE_SCALE, rate, false),
                1 | 3 => {
                    amount = amount.min(crate::math::mul_div(&e, old, rate, PRICE_SCALE, false));
                    crate::math::mul_div(&e, amount, PRICE_SCALE, rate, true)
                }
                4 => crate::math::mul_div(&e, amount, PRICE_SCALE, rate, true),
                5 => {
                    amount = amount.min(crate::math::mul_div(&e, old, rate, PRICE_SCALE, true));
                    crate::math::mul_div(&e, amount, PRICE_SCALE, rate, false)
                }
                _ => panic!("invalid request"),
            };
            let next = if matches!(r.request_type, 0 | 2 | 4) {
                old + shares
            } else {
                old - shares
            };
            assert!(next >= 0);
            if next == 0 {
                map.remove(i);
            } else {
                map.set(i, next);
            }
            if matches!(r.request_type, 0 | 2 | 5) {
                token.transfer(&spender, &e.current_contract_address(), &amount);
            } else {
                token.transfer(&e.current_contract_address(), &to, &amount);
            }
        }
        e.storage().persistent().set(&from, &p);
        p
    }
}
#[contract]
pub struct RouterFixture;
#[contractimpl]
impl RouterFixture {
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
        if let Some(hop) = e
            .storage()
            .instance()
            .get::<_, Address>(&symbol_short!("hop"))
        {
            HopFixtureClient::new(&e, &hop).swap(
                &to,
                &token_in,
                &distribution.get(0).unwrap().path.get(1).unwrap(),
                &token_out,
                &amount_in,
            );
            return vec![&e, vec![&e, amount_in, amount_in]];
        }
        let take = e
            .storage()
            .instance()
            .get(&symbol_short!("take"))
            .unwrap_or(amount_in);
        TokenClient::new(&e, &token_in).transfer(&to, &e.current_contract_address(), &take);
        let output = e
            .storage()
            .instance()
            .get(&symbol_short!("output"))
            .unwrap_or(amount_in);
        if e.storage()
            .instance()
            .get(&symbol_short!("reenter"))
            .unwrap_or(false)
        {
            VaultClient::new(&e, &to).collect_fees();
        }
        TokenClient::new(&e, &token_out).transfer(&e.current_contract_address(), &to, &output);
        let _ = amount_out_min;
        vec![&e, vec![&e, amount_in, output]]
    }
}

pub struct Fixture {
    pub e: Env,
    pub vault: Address,
    pub admin: Address,
    pub user: Address,
    pub executor: Address,
    pub guardian: Address,
    pub recipient: Address,
    pub usdc: Address,
    pub xlm: Address,
    pub reserve: Address,
    pub risk: Address,
    pub oracle: Address,
    pub router: Address,
    pub pool: Address,
}
impl Fixture {
    pub fn new(pool: bool, borrowing: bool) -> Self {
        let e = Env::default();
        e.cost_estimate().budget().reset_unlimited();
        e.ledger().with_mut(|l| {
            l.timestamp = 1_000_000;
            l.sequence_number = 100;
            l.max_entry_ttl = 3_110_400;
            l.min_persistent_entry_ttl = 3_000_000;
        });
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let user = Address::generate(&e);
        let executor = Address::generate(&e);
        let guardian = Address::generate(&e);
        let recipient = Address::generate(&e);
        let usdc = e
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let xlm = e
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let reserve = e
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let risk = e
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let oracle = e.register(OracleFixture, ());
        let router = e.register(RouterFixture, ());
        let pool_addr = e.register(
            PoolFixture,
            (
                vec![&e, usdc.clone(), xlm.clone(), risk.clone()],
                oracle.clone(),
            ),
        );
        let mut assets = std::vec![
            Asset {
                address: usdc.clone(),
                decimals: 7,
                kind: AssetKind::Settlement
            },
            Asset {
                address: xlm.clone(),
                decimals: 7,
                kind: AssetKind::Xlm
            },
            Asset {
                address: reserve.clone(),
                decimals: 7,
                kind: AssetKind::Reserve
            },
            Asset {
                address: risk.clone(),
                decimals: 7,
                kind: AssetKind::Risk
            }
        ];
        assets.sort_by(|a, b| a.address.cmp(&b.address));
        let assets = Vec::from_slice(&e, &assets);
        for a in assets.iter() {
            let t = StellarAssetClient::new(&e, &a.address);
            t.mint(&admin, &(SEED_ASSETS * 1000));
            t.mint(&user, &(SEED_ASSETS * 1000));
            t.mint(&router, &(SEED_ASSETS * 1000));
            t.mint(&pool_addr, &(SEED_ASSETS * 1000));
        }
        let config = Config {
            assets,
            usdc: usdc.clone(),
            oracle: oracle.clone(),
            router: router.clone(),
            route_contracts: vec![&e, router.clone()],
            pool: if pool { Some(pool_addr.clone()) } else { None },
            pool_assets: if pool {
                vec![&e, usdc.clone(), xlm.clone(), risk.clone()]
            } else {
                vec![&e]
            },
            admin: admin.clone(),
            executor: executor.clone(),
            guardian: guardian.clone(),
            fees: Fees {
                management_bps: 100,
                performance_bps: 1000,
                recipient: recipient.clone(),
            },
            borrowing,
            max_price_age: 300,
            max_divergence_bps: 100,
            max_slippage_bps: 100,
            max_leg_bps: 2000,
            max_turnover_bps: 10000,
            max_loss_bps: 200,
            cooldown: 1,
            max_debt_bps: if borrowing { 5000 } else { 0 },
            min_health_bps: 12500,
        };
        let vault = e.register(Vault, (config, admin.clone()));
        e.mock_auths(&[]);
        Self {
            e,
            vault,
            admin,
            user,
            executor,
            guardian,
            recipient,
            usdc,
            xlm,
            reserve,
            risk,
            oracle,
            router,
            pool: pool_addr,
        }
    }
    pub fn client(&self) -> VaultClient<'_> {
        VaultClient::new(&self.e, &self.vault)
    }
    pub fn auth(&self, who: &Address, function: &str, args: Vec<soroban_sdk::Val>) {
        self.e.mock_auths(&[MockAuth {
            address: who,
            invoke: &MockAuthInvoke {
                contract: &self.vault,
                fn_name: function,
                args,
                sub_invokes: &[],
            },
        }]);
    }
    pub fn deposit(&self, amount: i128) -> i128 {
        self.e.mock_all_auths();
        let s = self
            .client()
            .deposit(&amount, &self.user, &self.user, &self.user);
        self.e.mock_auths(&[]);
        s
    }
    pub fn donate(&self, asset: &Address, amount: i128) {
        self.e.mock_all_auths();
        StellarAssetClient::new(&self.e, asset).mint(&self.vault, &amount);
        self.e.mock_auths(&[]);
    }
    pub fn mark(&self, asset: &Address, price: i128, valid: bool) {
        self.e.as_contract(&self.oracle, || {
            self.e.storage().instance().set(
                asset,
                &Mark {
                    price,
                    reference: price,
                    timestamp: self.e.ledger().timestamp(),
                    ready: valid,
                },
            )
        });
    }
    pub fn advance(&self, seconds: u64, ledgers: u32) {
        self.e.ledger().with_mut(|l| {
            l.timestamp += seconds;
            l.sequence_number += ledgers;
        });
    }
    pub fn target(&self, yield_bps: u32) -> Target {
        let e = &self.e;
        let c = self.client();
        let assets = c.config().assets;
        let y = 9500 * yield_bps / 10000;
        let mut addresses = Vec::new(e);
        let mut weights = Vec::new(e);
        for a in assets.iter() {
            addresses.push_back(a.address);
            weights.push_back(match a.kind {
                AssetKind::Settlement => 250 + y,
                AssetKind::Xlm => 250,
                AssetKind::Reserve => 9500 - y,
                _ => 0,
            });
        }
        Target {
            network: e.ledger().network_id(),
            vault: self.vault.clone(),
            epoch: 1,
            expiry: e.ledger().timestamp() + 300,
            model: soroban_sdk::BytesN::from_array(e, &[1; 32]),
            dataset: soroban_sdk::BytesN::from_array(e, &[2; 32]),
            holdings: c.holdings_hash(),
            assets: addresses,
            weights,
            yield_bps,
            equity: c.total_assets(),
            supply: c.state().supply,
        }
    }
    pub fn swap(&self, input: &Address, output: &Address, amount: i128) -> Swap {
        let e = &self.e;
        Swap {
            token_in: input.clone(),
            token_out: output.clone(),
            amount,
            minimum: amount * 99 / 100,
            distribution: vec![
                e,
                DexDistribution {
                    protocol_id: Protocol::Soroswap,
                    path: vec![e, input.clone(), output.clone()],
                    parts: 1,
                    bytes: None,
                },
            ],
            auth: vec![
                e,
                RouteAuth::Transfer(input.clone(), self.router.clone(), amount),
            ],
        }
    }
    pub fn plan(&self, actions: Vec<Action>, yield_bps: u32) -> Plan {
        let t = self.target(yield_bps);
        self.auth(
            &self.executor,
            "publish_target",
            (t.clone(),).into_val(&self.e),
        );
        let hash = self.client().publish_target(&t);
        Plan {
            target: hash,
            epoch: t.epoch,
            nonce: self.client().state().nonce,
            version: self.client().state().version,
            deadline: t.expiry,
            actions,
        }
    }
    pub fn positions(&self, supply: i128, collateral: i128, debt: i128) {
        self.e.as_contract(&self.pool, || {
            let mut p = Positions {
                supply: map![&self.e],
                collateral: map![&self.e],
                liabilities: map![&self.e],
            };
            if supply > 0 {
                p.supply.set(0, supply);
            }
            if collateral > 0 {
                p.collateral.set(0, collateral);
            }
            if debt > 0 {
                p.liabilities.set(1, debt);
            }
            self.e.storage().persistent().set(&self.vault, &p);
        });
    }
}

#[contract]
pub struct HopFixture;
#[contractimpl]
impl HopFixture {
    pub fn swap(
        e: Env,
        owner: Address,
        input: Address,
        middle: Address,
        output: Address,
        amount: i128,
    ) {
        owner.require_auth();
        let this = e.current_contract_address();
        TokenClient::new(&e, &input).transfer(&owner, &this, &amount);
        let short: i128 = e
            .storage()
            .instance()
            .get(&symbol_short!("short"))
            .unwrap_or(0);
        TokenClient::new(&e, &middle).transfer(&this, &owner, &(amount - short));
        TokenClient::new(&e, &middle).transfer(&owner, &this, &amount);
        TokenClient::new(&e, &output).transfer(&this, &owner, &amount);
    }
}
