extern crate std;
use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Events, Ledger},
    vec,
};

#[contract]
struct FeedFixture;
#[contractimpl]
impl FeedFixture {
    pub fn base(e: Env) -> FeedAsset {
        e.storage().instance().get(&symbol_short!("base")).unwrap()
    }
    pub fn decimals(e: Env) -> u32 {
        e.storage()
            .instance()
            .get(&symbol_short!("decimals"))
            .unwrap_or(14)
    }
    pub fn resolution(_e: Env) -> u32 {
        300
    }
    pub fn lastprice(e: Env, asset: FeedAsset) -> Option<PriceData> {
        e.storage().instance().get(&asset)
    }
}
#[contract]
struct PairFixture;
#[contractimpl]
impl PairFixture {
    pub fn token_0(e: Env) -> Address {
        e.storage().instance().get(&0u32).unwrap()
    }
    pub fn token_1(e: Env) -> Address {
        e.storage().instance().get(&1u32).unwrap()
    }
    pub fn get_reserves(e: Env) -> (i128, i128) {
        e.storage().instance().get(&2u32).unwrap()
    }
}
struct Fixture {
    e: Env,
    provider: Address,
    c: Config,
}
impl Fixture {
    fn new(conversion: bool) -> Self {
        let e = Env::default();
        e.cost_estimate().budget().reset_unlimited();
        e.ledger().with_mut(|l| {
            l.timestamp = 1000;
            l.sequence_number = 100;
            l.max_entry_ttl = 3_110_400;
        });
        let admin = Address::generate(&e);
        let usdc = e
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let asset = e.register_stellar_asset_contract_v2(admin).address();
        let feed = e.register(FeedFixture, ());
        let pool = e.register(PairFixture, ());
        let base = if conversion {
            FeedAsset::Other(symbol_short!("USD"))
        } else {
            FeedAsset::Stellar(usdc.clone())
        };
        e.as_contract(&feed, || {
            e.storage().instance().set(&symbol_short!("base"), &base)
        });
        e.as_contract(&pool, || {
            e.storage().instance().set(&0u32, &asset);
            e.storage().instance().set(&1u32, &usdc);
        });
        let numerator = Feed {
            contract: feed.clone(),
            asset: FeedAsset::Stellar(asset.clone()),
            base: base.clone(),
            decimals: 14,
            resolution: 300,
            max_age: 300,
        };
        let denominator = conversion.then(|| Feed {
            contract: feed,
            asset: FeedAsset::Stellar(usdc.clone()),
            base,
            decimals: 14,
            resolution: 300,
            max_age: 300,
        });
        let c = Config {
            usdc,
            sources: vec![
                &e,
                Source {
                    asset,
                    decimals: 7,
                    numerator,
                    denominator: Vec::from_slice(&e, denominator.as_slice()),
                    pool,
                    asset_is_token0: true,
                    min_asset_reserve: 10_000_000,
                    min_usdc_reserve: 10_000_000,
                    min_spacing: 5,
                    max_gap: 60,
                    max_age: 300,
                    max_skew: 300,
                    divergence_bps: 100,
                },
            ],
        };
        let provider = e.register(Pricing, (c.clone(),));
        let f = Self { e, provider, c };
        f.price(100 * SCALE, 1000);
        if let Some(d) = f.s().denominator.first() {
            f.write_feed(&d, SCALE * 100, 1000);
        }
        f.reserves(100 * 10_000_000);
        f
    }
    fn client(&self) -> PricingClient<'_> {
        PricingClient::new(&self.e, &self.provider)
    }
    fn s(&self) -> Source {
        self.c.sources.get(0).unwrap()
    }
    fn asset(&self) -> Address {
        self.s().asset
    }
    fn write_feed(&self, f: &Feed, price: i128, timestamp: u64) {
        self.e.as_contract(&f.contract, || {
            self.e
                .storage()
                .instance()
                .set(&f.asset, &PriceData { price, timestamp })
        });
    }
    fn price(&self, price: i128, timestamp: u64) {
        self.write_feed(&self.s().numerator, price * 100, timestamp);
    }
    fn reserves(&self, usdc: i128) {
        self.e.as_contract(&self.s().pool, || {
            self.e
                .storage()
                .instance()
                .set(&2u32, &(10_000_000i128, usdc))
        });
    }
    fn advance(&self, seconds: u64) {
        self.e.ledger().with_mut(|l| {
            l.timestamp += seconds;
            l.sequence_number += 1;
        });
    }
    fn collect(&self, prices: [i128; 3], gaps: [u64; 2]) {
        for i in 0..3 {
            if i > 0 {
                self.advance(gaps[i - 1]);
            }
            self.reserves(prices[i] * 10_000_000);
            self.client().observe(&self.asset());
        }
        self.advance(1);
    }
    fn error(&self, err: Error) {
        assert_eq!(self.client().diagnose(&self.asset()).error, err as u32);
        assert!(self.client().try_mark(&self.asset()).is_err());
    }
}

#[test]
fn temporal_mean_only_and_pending_cannot_evict() {
    let f = Fixture::new(false);
    f.error(Error::Bootstrap);
    f.collect([90, 100, 110], [5, 5]);
    let m = f.client().mark(&f.asset());
    assert_eq!(
        (m.price, m.reference, m.timestamp),
        (100 * SCALE, 100 * SCALE, 1000)
    );
    assert_eq!(f.client().diagnose(&f.asset()).samples.len(), 3);
    f.advance(5);
    f.reserves(1000 * 10_000_000);
    f.client().observe(&f.asset());
    assert_eq!(f.client().mark(&f.asset()), m);
    assert!(f.client().try_observe(&f.asset()).is_err());
    f.advance(1);
    f.error(Error::Divergence);
}
#[test]
fn unequal_spacing_equal_prices_and_rounding() {
    let f = Fixture::new(false);
    f.collect([90, 100, 110], [5, 10]);
    let d = f.client().diagnose(&f.asset());
    assert_eq!(d.mark.reference, 101_666_666_666_666);
    assert_eq!(d.error, Error::Divergence as u32);
    f.advance(61);
    f.collect([100, 100, 100], [5, 5]);
    assert_eq!(f.client().mark(&f.asset()).reference, 100 * SCALE);
    let mut samples = f.client().diagnose(&f.asset()).samples;
    for (i, price) in [1, 1, 2].iter().enumerate() {
        let mut p = samples.get(i as u32).unwrap();
        p.price = *price;
        samples.set(i as u32, p);
    }
    assert_eq!(math::twap(&f.e, &samples), Ok(1));
    let mut p = samples.get(1).unwrap();
    p.timestamp = samples.get(0).unwrap().timestamp;
    samples.set(1, p);
    assert_eq!(math::twap(&f.e, &samples), Err(Error::Invalid));
    assert_eq!(math::twap(&f.e, &Vec::new(&f.e)), Err(Error::Bootstrap));
}
#[test]
fn conversion_nonunit_denominator_and_all_leg_age_checks() {
    let f = Fixture::new(true);
    let den = f.s().denominator.first().unwrap();
    f.write_feed(&den, 125_000_000_000_000, 1000);
    f.price(125 * SCALE, 1000);
    f.collect([100, 100, 100], [5, 5]);
    assert_eq!(f.client().mark(&f.asset()).price, 100 * SCALE);
    f.write_feed(&den, 125_000_000_000_000, 1012);
    f.error(Error::Future);
    f.write_feed(&den, 125_000_000_000_000, 710);
    f.error(Error::Stale);
    f.write_feed(&den, 0, 1000);
    f.error(Error::Invalid);
    f.e.as_contract(&den.contract, || {
        f.e.storage().instance().remove(&den.asset)
    });
    f.error(Error::Missing);
    f.write_feed(&den, 125_000_000_000_000, 1000);
    assert!(f.client().mark(&f.asset()).ready);
}
#[test]
fn source_failures_do_not_use_cached_marks() {
    let f = Fixture::new(false);
    f.collect([100, 100, 100], [5, 5]);
    for (price, time, error) in [
        (0, 1000, Error::Invalid),
        (-1, 1000, Error::Invalid),
        (100 * SCALE, 1012, Error::Future),
        (100 * SCALE, 710, Error::Stale),
    ] {
        f.price(price, time);
        f.error(error);
    }
    f.price(100 * SCALE, 1000);
    f.e.as_contract(&f.s().numerator.contract, || {
        f.e.storage()
            .instance()
            .set(&symbol_short!("decimals"), &13u32)
    });
    f.error(Error::Identity);
    f.e.as_contract(&f.s().numerator.contract, || {
        f.e.storage()
            .instance()
            .set(&symbol_short!("decimals"), &14u32)
    });
    assert!(f.client().mark(&f.asset()).ready);
    assert!(f.client().try_mark(&Address::generate(&f.e)).is_err());
    assert_eq!(f.client().mark(&f.c.usdc).price, SCALE);
    // Missing source method fails closed, with a bounded reason code.
    let bad = Feed {
        contract: f.s().pool,
        ..f.s().numerator
    };
    assert_eq!(sources::feed(&f.e, &bad), Err(Error::Invocation));
}
#[test]
fn depth_order_spacing_and_gap_recovery() {
    let f = Fixture::new(false);
    f.reserves(0);
    assert!(f.client().try_observe(&f.asset()).is_err());
    f.reserves(100 * 10_000_000);
    f.client().observe(&f.asset());
    f.advance(4);
    assert!(f.client().try_observe(&f.asset()).is_err());
    f.advance(1);
    f.client().observe(&f.asset());
    f.advance(5);
    f.client().observe(&f.asset());
    f.error(Error::Bootstrap);
    f.advance(1);
    assert!(f.client().mark(&f.asset()).ready);
    f.advance(61);
    f.client().observe(&f.asset());
    f.error(Error::Bootstrap);
    f.advance(5);
    f.client().observe(&f.asset());
    f.advance(5);
    f.client().observe(&f.asset());
    f.advance(1);
    assert!(f.client().mark(&f.asset()).ready);
    f.e.as_contract(&f.s().pool, || {
        f.e.storage().instance().set(&0u32, &f.c.usdc)
    });
    f.advance(5);
    assert!(f.client().try_observe(&f.asset()).is_err());
}
#[test]
fn age_skew_and_ttl_loss() {
    let f = Fixture::new(false);
    // Isolate the oldest-sample age boundary from the separately tested window-end gap.
    f.e.as_contract(&f.provider, || {
        let mut c = f.c.clone();
        let mut s = f.s();
        s.max_gap = 300;
        c.sources.set(0, s);
        f.e.storage().instance().set(&Key::Config, &c);
    });
    f.collect([100, 100, 100], [5, 5]);
    f.advance(289);
    assert!(f.client().mark(&f.asset()).ready); // Exactly 300.
    f.advance(1);
    f.error(Error::Stale);
    f.price(100 * SCALE, 1301);
    f.error(Error::Stale); // Samples independently stale.
    f.client().upkeep();
    f.e.as_contract(&f.provider, || {
        f.e.storage().temporary().remove(&Key::History(f.asset()))
    });
    f.error(Error::Bootstrap);
    f.collect([100, 100, 100], [5, 5]);
    assert!(f.client().mark(&f.asset()).ready);
    let hash = f.client().config_hash();
    let saved_config = f.client().config();
    f.client().upkeep();
    assert_eq!(hash, f.client().config_hash());
    assert_eq!(f.client().config(), saved_config);
    // Source skew is checked independently of each age.
    f.e.as_contract(&f.provider, || {
        let mut c = f.c.clone();
        let mut s = f.s();
        s.max_skew = 10;
        c.sources.set(0, s);
        f.e.storage().instance().set(&Key::Config, &c);
    });
    f.price(100 * SCALE, f.e.ledger().timestamp());
    f.error(Error::Skew);
    f.e.as_contract(&f.provider, || {
        f.e.storage().instance().remove(&Key::Config)
    });
    assert!(f.client().try_mark(&f.asset()).is_err());
}
#[test]
fn decimal_vectors_wide_arithmetic_and_exact_divergence() {
    let e = Env::default();
    assert_eq!(
        math::ratio(&e, 125_000_000, 6, 1_250_000_000, 9),
        Ok(100 * SCALE)
    );
    assert_eq!(
        math::ratio(&e, 107_451_712_398_738, 14, 1, 0),
        Ok(1_074_517_123_987)
    );
    assert_eq!(
        math::ratio(&e, 1_000_000_000_000_000_000, 18, 1, 0),
        Ok(SCALE)
    );
    assert_eq!(math::ratio(&e, i128::MAX, 18, i128::MAX, 18), Ok(SCALE));
    assert_eq!(math::ratio(&e, i128::MAX, 0, 1, 18), Err(Error::Overflow));
    assert_eq!(math::ratio(&e, 1, 18, i128::MAX, 0), Err(Error::Invalid));
    assert_eq!(math::ratio(&e, 1, 19, 1, 0), Err(Error::Invalid));
    assert!(math::within(&e, 10000, 10100, 100));
    assert!(!math::within(&e, 10000, 10101, 100));
    assert!(math::within(&e, 10000, 9900, 100));
    assert!(!math::within(&e, 10000, 9899, 100));
}
#[test]
fn reversed_pool_and_metadata_mismatch() {
    let f = Fixture::new(false);
    let mut s = f.s();
    s.asset_is_token0 = false;
    f.e.as_contract(&s.pool, || {
        f.e.storage().instance().set(&0u32, &f.c.usdc);
        f.e.storage().instance().set(&1u32, &s.asset);
        f.e.storage()
            .instance()
            .set(&2u32, &(1_000_000_000i128, 10_000_000i128));
    });
    assert_eq!(
        sources::pool(&f.e, &f.c.usdc, &s).unwrap().price,
        100 * SCALE
    );
    s.decimals = 8;
    assert_eq!(sources::pool(&f.e, &f.c.usdc, &s), Err(Error::Identity));
}
#[test]
fn configuration_rejects_invalid_limits_and_conversion_identity() {
    let f = Fixture::new(false);
    for case in 0..9 {
        let mut c = f.c.clone();
        let mut s = f.s();
        match case {
            0 => s.min_spacing = 0,
            1 => s.max_age = 3601,
            2 => s.divergence_bps = 501,
            3 => s.min_usdc_reserve = 0,
            4 => s.numerator.decimals = 19,
            5 => s.numerator.base = FeedAsset::Other(symbol_short!("USD")),
            6 => s.numerator.resolution = 0,
            7 => s.max_skew = 1,
            _ => s.asset = c.usdc.clone(),
        }
        c.sources.set(0, s);
        // Call the constructor inside a fresh contract frame to inspect typed errors.
        f.e.as_contract(&f.provider, || {
            assert_eq!(Pricing::__constructor(f.e.clone(), c), Err(Error::Config))
        });
    }
    let mut c = f.c.clone();
    c.sources.push_back(f.s());
    f.e.as_contract(&f.provider, || {
        assert_eq!(Pricing::__constructor(f.e.clone(), c), Err(Error::Config))
    });
    let mut c = f.c.clone();
    c.sources = Vec::new(&f.e);
    f.e.as_contract(&f.provider, || {
        assert_eq!(Pricing::__constructor(f.e.clone(), c), Err(Error::Config))
    });
    let mut c = f.c.clone();
    let mut s = f.s();
    s.denominator = vec![&f.e, s.numerator.clone()];
    c.sources.set(0, s);
    f.e.as_contract(&f.provider, || {
        assert_eq!(Pricing::__constructor(f.e.clone(), c), Err(Error::Config))
    });
}

fn funded_vault(f: &Fixture) -> (Address, Address) {
    use etesia_vault::{
        types::{Asset, AssetKind, Config as VaultConfig, Fees, SEED_ASSETS},
        Vault,
    };
    use soroban_sdk::token::StellarAssetClient;
    let e = &f.e;
    e.mock_all_auths();
    let admin = Address::generate(e);
    let user = Address::generate(e);
    // SAC admin is provided by testutils for issuance; ordinary token transfers remain real.
    e.mock_all_auths();
    StellarAssetClient::new(e, &f.c.usdc).mint(&admin, &(SEED_ASSETS * 10));
    StellarAssetClient::new(e, &f.c.usdc).mint(&user, &(SEED_ASSETS * 10));
    let xlm = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let mut assets = std::vec![
        Asset {
            address: f.c.usdc.clone(),
            decimals: 7,
            kind: AssetKind::Settlement
        },
        Asset {
            address: xlm,
            decimals: 7,
            kind: AssetKind::Xlm
        },
        Asset {
            address: f.asset(),
            decimals: 7,
            kind: AssetKind::Reserve
        }
    ];
    assets.sort_by(|a, b| a.address.cmp(&b.address));
    let c = VaultConfig {
        assets: Vec::from_slice(e, &assets),
        usdc: f.c.usdc.clone(),
        oracle: f.provider.clone(),
        router: Address::generate(e),
        route_contracts: Vec::new(e),
        pool: None,
        pool_assets: Vec::new(e),
        admin: admin.clone(),
        executor: admin.clone(),
        guardian: admin.clone(),
        fees: Fees {
            management_bps: 100,
            performance_bps: 1000,
            recipient: admin.clone(),
        },
        borrowing: false,
        max_price_age: 300,
        max_divergence_bps: 100,
        max_slippage_bps: 100,
        max_leg_bps: 2000,
        max_turnover_bps: 10000,
        max_loss_bps: 200,
        cooldown: 1,
        max_debt_bps: 0,
        min_health_bps: 12500,
    };
    let vault = e.register(Vault, (c, admin.clone()));
    (vault, user)
}

#[test]
fn new_vault_uses_provider_and_failures_preserve_state() {
    use etesia_vault::{types::SEED_ASSETS, VaultClient};
    use soroban_sdk::token::StellarAssetClient;
    let f = Fixture::new(false);
    let e = &f.e;
    let (vault, user) = funded_vault(&f);
    let v = VaultClient::new(e, &vault);
    // Unheld unready assets do not block deposits.
    let shares = v.deposit(&10_000_000, &user, &user, &user);
    StellarAssetClient::new(e, &f.asset()).mint(&vault, &10_000_000);
    let state = v.state();
    let holdings = v.holdings();
    let events = e.events().all();
    let balance = v.balance(&user);
    assert!(v.try_deposit(&10_000_000, &user, &user, &user).is_err());
    assert!(v.try_redeem(&shares, &user, &user, &user).is_err());
    assert_eq!(v.state(), state);
    assert_eq!(v.holdings(), holdings);
    assert_eq!(v.balance(&user), balance);
    assert_eq!(e.events().all(), events);
    f.collect([90, 100, 110], [5, 5]);
    assert_eq!(
        v.total_assets(),
        SEED_ASSETS + 10_000_000 + 100 * 10_000_000
    );
    v.redeem(&shares, &user, &user, &user);
    f.advance(301);
    assert!(v.try_total_assets().is_err());
    assert!(!v.state().paused);
    assert_eq!(v.max_deposit(&user), 0);
}

#[test]
fn python_rust_exact_rational_vectors() {
    let e = Env::default();
    for line in include_str!("../../../fixtures/d3-ratio.csv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let v: std::vec::Vec<i128> = line.split(',').map(|n| n.parse().unwrap()).collect();
        assert_eq!(
            math::ratio(&e, v[0], v[1] as u32, v[2], v[3] as u32),
            Ok(v[4])
        );
    }
    for line in include_str!("../../../fixtures/d3-twap.csv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let v: std::vec::Vec<i128> = line.split(',').map(|n| n.parse().unwrap()).collect();
        let mut samples = Vec::new(&e);
        for i in 0..3 {
            samples.push_back(Observation {
                price: v[i],
                timestamp: v[i + 3] as u64,
                ledger: i as u32,
                asset_reserve: 1,
                usdc_reserve: 1,
            });
        }
        assert_eq!(math::twap(&e, &samples), Ok(v[6]));
    }
}

#[test]
fn source_recovery_does_not_clear_guardian_pause_and_basket_survives() {
    use etesia_vault::VaultClient;
    use soroban_sdk::token::StellarAssetClient;
    let f = Fixture::new(false);
    let (vault, user) = funded_vault(&f);
    let v = VaultClient::new(&f.e, &vault);
    let shares = v.deposit(&100_000_000, &user, &user, &user);
    StellarAssetClient::new(&f.e, &f.asset()).mint(&vault, &10_000_000);
    f.collect([100, 100, 100], [5, 5]);
    v.pause();
    f.price(0, 1000);
    assert!(v.try_total_assets().is_err());
    f.price(100 * SCALE, 1000);
    assert!(v.total_assets() > 0);
    assert!(v.state().paused);
    assert!(v.try_deposit(&10_000_000, &user, &user, &user).is_err());
    f.advance(301);
    assert!(v.try_redeem(&shares, &user, &user, &user).is_err());
    let high_water = v.state().high_water;
    let out = v.redeem_in_kind(
        &shares,
        &user,
        &user,
        &vec![&f.e, 0, 0, 0],
        &(f.e.ledger().timestamp() + 10),
    );
    assert!(out.iter().any(|n| n > 0));
    assert_eq!(v.balance(&user), 0);
    assert_eq!(v.state().high_water, high_water);
}

#[test]
fn feed_update_timing_exposes_residual_stale_mark_arbitrage() {
    use etesia_vault::VaultClient;
    use soroban_sdk::token::StellarAssetClient;
    let f = Fixture::new(false);
    let (vault, user) = funded_vault(&f);
    let v = VaultClient::new(&f.e, &vault);
    StellarAssetClient::new(&f.e, &f.asset()).mint(&vault, &100_000_000);
    // Both layers can lag the economic price together; a mean-only breaker is not a latency defense.
    let mut gains = std::vec![];
    for old in [100, 110, 120] {
        f.advance(61);
        f.price(old * SCALE, f.e.ledger().timestamp());
        f.collect([old, old, old], [5, 5]);
        let shares = v.deposit(&100_000_000, &user, &user, &user);
        f.advance(61);
        f.price((old + 10) * SCALE, f.e.ledger().timestamp());
        f.collect([old + 10, old + 10, old + 10], [5, 5]);
        let received = v.redeem(&shares, &user, &user, &user);
        assert!(received > 100_000_000);
        gains.push(received - 100_000_000);
    }
    std::println!(
        "three lagging-mark entry/exit gains in USDC atoms: {:?}",
        gains
    );
}

#[test]
fn reference_symbol_conversion_and_history_corruption_fail_closed() {
    let f = Fixture::new(true);
    let mut c = f.c.clone();
    let mut s = f.s();
    let mut d = s.denominator.first().unwrap();
    d.asset = FeedAsset::Other(symbol_short!("USDC"));
    s.denominator = vec![&f.e, d.clone()];
    c.sources.set(0, s);
    let provider = f.e.register(Pricing, (c,));
    assert_eq!(
        PricingClient::new(&f.e, &provider)
            .config()
            .sources
            .get(0)
            .unwrap()
            .denominator
            .first()
            .unwrap()
            .asset,
        d.asset
    );
    f.collect([100, 100, 100], [5, 5]);
    for case in 0..5 {
        let mut samples = f.client().diagnose(&f.asset()).samples;
        // Restore fresh baseline before each adversarial storage case.
        for i in 0..3 {
            let mut p = samples.get(i).unwrap();
            p.timestamp = 1000 + u64::from(i) * 5;
            p.ledger = 100 + i;
            p.asset_reserve = 10_000_000;
            samples.set(i, p);
        }
        let mut p = samples.get(1).unwrap();
        match case {
            0 => p.ledger = 100,
            1 => p.timestamp = 1000,
            2 => p.timestamp = 1001,
            3 => p.asset_reserve = 0,
            _ => p.ledger = 1000,
        };
        samples.set(1, p);
        f.e.as_contract(&f.provider, || {
            f.e.storage().temporary().set(
                &Key::History(f.asset()),
                &History {
                    completed: samples,
                    pending: Vec::new(&f.e),
                },
            )
        });
        assert!(f.client().try_mark(&f.asset()).is_err());
    }
}

#[test]
fn restoring_configuration_cannot_restore_expired_temporary_history() {
    use soroban_sdk::{
        testutils::storage::{Instance, Temporary},
        xdr::{ContractDataDurability, LedgerEntryData},
    };
    use std::string::ToString;
    let f = Fixture::new(false);
    f.collect([100, 100, 100], [5, 5]);
    f.client().upkeep();
    f.e.as_contract(&f.provider, || {
        assert!(f.e.storage().instance().get_ttl() >= TTL / 2);
        assert!(f.e.storage().temporary().get_ttl(&Key::History(f.asset())) >= TTL - 3);
    });
    let expected = f.client().config_hash().to_array();
    let mut snapshot = f.e.to_ledger_snapshot();
    snapshot.sequence_number += TTL + 1;
    snapshot.timestamp += 86400;
    // Temporary entries are deleted, not archived. Only persistent payloads are restored.
    snapshot.ledger_entries.retain(|(_, (entry, _))| !matches!(&entry.data, LedgerEntryData::ContractData(data) if data.durability==ContractDataDurability::Temporary));
    for (_, (_, ttl)) in snapshot.ledger_entries.iter_mut() {
        if ttl.is_some() {
            *ttl = Some(snapshot.sequence_number + TTL);
        }
    }
    let restored = Env::from_ledger_snapshot(snapshot);
    restored.cost_estimate().budget().reset_unlimited();
    let provider = Address::from_str(&restored, &f.provider.to_string().to_string());
    restored.as_contract(&provider, || {
        assert_eq!(
            Pricing::config_hash(restored.clone()).unwrap().to_array(),
            expected
        );
        let asset = config(&restored).unwrap().sources.get(0).unwrap().asset;
        let h = history(&restored, &asset);
        assert!(h.completed.is_empty() && h.pending.is_empty());
    });
}

#[test]
fn overdue_collection_cannot_evict_a_usable_window_in_its_ledger() {
    let f = Fixture::new(false);
    f.collect([100, 100, 100], [5, 5]);
    f.advance(59);
    assert!(f.client().mark(&f.asset()).ready);
    f.advance(1);
    f.error(Error::Stale);
    f.client().observe(&f.asset());
    f.error(Error::Bootstrap);
}
