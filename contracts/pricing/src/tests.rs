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
        let base = if conversion {
            FeedAsset::Other(symbol_short!("USD"))
        } else {
            FeedAsset::Stellar(usdc.clone())
        };
        e.as_contract(&feed, || {
            e.storage().instance().set(&symbol_short!("base"), &base)
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
                    numerator,
                    denominator: Vec::from_slice(&e, denominator.as_slice()),
                },
            ],
        };
        let provider = e.register(Pricing, (c.clone(),));
        let f = Self { e, provider, c };
        f.price(100 * SCALE, 1000);
        if let Some(d) = f.s().denominator.first() {
            f.write_feed(&d, SCALE * 100, 1000);
        }
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
    fn advance(&self, seconds: u64) {
        self.e.ledger().with_mut(|l| {
            l.timestamp += seconds;
            l.sequence_number += 1;
        });
    }
    fn error(&self, err: Error) {
        let d = self.client().diagnose(&self.asset());
        assert_eq!(d.error, err as u32);
        assert_eq!(
            d.mark,
            Mark {
                price: 0,
                timestamp: 0
            }
        );
        assert!(self.client().try_mark(&self.asset()).is_err());
    }
}

#[test]
fn direct_mark_is_the_reflector_price() {
    let f = Fixture::new(false);
    let d = f.client().diagnose(&f.asset());
    assert_eq!((d.error, d.numerator.len(), d.denominator.len()), (0, 1, 0));
    assert_eq!(
        f.client().mark(&f.asset()),
        Mark {
            price: 100 * SCALE,
            timestamp: 1000
        }
    );
    assert_eq!(d.mark, f.client().mark(&f.asset()));
    f.price(110 * SCALE, 1000);
    assert_eq!(f.client().mark(&f.asset()).price, 110 * SCALE);
}
#[test]
fn conversion_nonunit_denominator_and_all_leg_age_checks() {
    let f = Fixture::new(true);
    let den = f.s().denominator.first().unwrap();
    f.write_feed(&den, 125_000_000_000_000, 990);
    f.price(125 * SCALE, 1000);
    // The mark is only as fresh as its oldest leg.
    assert_eq!(
        f.client().mark(&f.asset()),
        Mark {
            price: 100 * SCALE,
            timestamp: 990
        }
    );
    assert_eq!(f.client().diagnose(&f.asset()).denominator.len(), 1);
    f.write_feed(&den, 125_000_000_000_000, 1012);
    f.error(Error::Future);
    f.write_feed(&den, 125_000_000_000_000, 699);
    f.error(Error::Stale);
    f.write_feed(&den, 0, 1000);
    f.error(Error::Invalid);
    f.e.as_contract(&den.contract, || {
        f.e.storage().instance().remove(&den.asset)
    });
    f.error(Error::Missing);
    f.write_feed(&den, 125_000_000_000_000, 1000);
    assert_eq!(f.client().mark(&f.asset()).price, 100 * SCALE);
    f.write_feed(&den, 1, 1000);
    f.price(i128::MAX / 100, 1000);
    f.error(Error::Overflow);
}
#[test]
fn source_failures_do_not_use_cached_marks() {
    let f = Fixture::new(false);
    for (price, time, error) in [
        (0, 1000, Error::Invalid),
        (-1, 1000, Error::Invalid),
        (100 * SCALE, 1012, Error::Future),
        (100 * SCALE, 699, Error::Stale),
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
    assert_eq!(f.client().mark(&f.asset()).price, 100 * SCALE);
    assert!(f.client().try_mark(&Address::generate(&f.e)).is_err());
    assert!(f.client().try_diagnose(&Address::generate(&f.e)).is_err());
    let usdc = f.client().mark(&f.c.usdc);
    assert_eq!((usdc.price, usdc.timestamp), (SCALE, 1000));
    // Missing source method fails closed, with a bounded reason code.
    let bad = Feed {
        contract: f.c.usdc.clone(),
        ..f.s().numerator
    };
    assert_eq!(sources::feed(&f.e, &bad), Err(Error::Invocation));
}
#[test]
fn age_boundary_and_config_loss() {
    let f = Fixture::new(false);
    f.advance(300);
    assert_eq!(f.client().mark(&f.asset()).timestamp, 1000); // Exactly 300.
    f.advance(1);
    f.error(Error::Stale);
    f.price(100 * SCALE, 1301);
    assert_eq!(f.client().mark(&f.asset()).timestamp, 1301);
    let hash = f.client().config_hash();
    assert_eq!(f.client().config(), f.c);
    assert_eq!(hash, f.client().config_hash());
    f.e.as_contract(&f.provider, || {
        f.e.storage().instance().remove(&Key::Config)
    });
    assert!(f.client().try_mark(&f.asset()).is_err());
}
#[test]
fn decimal_vectors_and_wide_arithmetic() {
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
}
fn rejects(f: &Fixture, c: Config) {
    // Call the constructor inside a fresh contract frame to inspect typed errors.
    f.e.as_contract(&f.provider, || {
        assert_eq!(Pricing::__constructor(f.e.clone(), c), Err(Error::Config))
    });
}
#[test]
fn configuration_rejects_invalid_limits_and_conversion_identity() {
    let f = Fixture::new(false);
    for case in 0..6 {
        let mut c = f.c.clone();
        let mut s = f.s();
        match case {
            0 => s.numerator.max_age = 0,
            1 => s.numerator.max_age = 3601,
            2 => s.numerator.decimals = 19,
            3 => s.numerator.base = FeedAsset::Other(symbol_short!("USD")),
            4 => s.numerator.resolution = 0,
            _ => s.asset = c.usdc.clone(),
        }
        c.sources.set(0, s);
        rejects(&f, c);
    }
    let mut c = f.c.clone();
    c.sources.push_back(f.s());
    rejects(&f, c);
    let mut c = f.c.clone();
    c.sources = Vec::new(&f.e);
    rejects(&f, c);
    let mut c = f.c.clone();
    let mut s = f.s();
    s.denominator = vec![&f.e, s.numerator.clone(), s.numerator.clone()];
    c.sources.set(0, s);
    rejects(&f, c);
    let mut c = f.c.clone();
    for _ in 0..7 {
        let mut s = f.s();
        s.asset = Address::generate(&f.e);
        c.sources.push_back(s);
    }
    rejects(&f, c);
    let f = Fixture::new(true);
    for case in 0..3 {
        let mut c = f.c.clone();
        let mut s = f.s();
        let mut d = s.denominator.first().unwrap();
        match case {
            0 => d.base = FeedAsset::Stellar(c.usdc.clone()),
            1 => d.asset = FeedAsset::Other(symbol_short!("USD")),
            _ => d.max_age = 0,
        }
        s.denominator = vec![&f.e, d];
        c.sources.set(0, s);
        rejects(&f, c);
    }
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
    // Unheld unpriceable assets do not block deposits.
    f.price(0, 1000);
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
    f.price(100 * SCALE, 1000);
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
    // Reflector can lag the economic price; age bounds are not a latency defense.
    let mut gains = std::vec![];
    for old in [100, 110, 120] {
        f.advance(61);
        f.price(old * SCALE, f.e.ledger().timestamp());
        let shares = v.deposit(&100_000_000, &user, &user, &user);
        f.advance(61);
        f.price((old + 10) * SCALE, f.e.ledger().timestamp());
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
fn usdc_symbol_conversion_leg_is_accepted() {
    let f = Fixture::new(true);
    let mut c = f.c.clone();
    let mut s = f.s();
    let mut d = s.denominator.first().unwrap();
    d.asset = FeedAsset::Other(symbol_short!("USDC"));
    s.denominator = vec![&f.e, d.clone()];
    c.sources.set(0, s);
    let provider = f.e.register(Pricing, (c,));
    let p = PricingClient::new(&f.e, &provider);
    assert_eq!(
        p.config()
            .sources
            .get(0)
            .unwrap()
            .denominator
            .first()
            .unwrap(),
        d
    );
    f.write_feed(&d, SCALE * 100, 1000);
    assert_eq!(p.mark(&f.asset()).price, 100 * SCALE);
}

#[test]
fn restored_snapshot_preserves_configuration() {
    use soroban_sdk::testutils::storage::Instance;
    use std::string::ToString;
    let f = Fixture::new(false);
    f.e.as_contract(&f.provider, || {
        assert!(f.e.storage().instance().get_ttl() >= TTL / 2);
    });
    let expected = f.client().config_hash().to_array();
    let mut snapshot = f.e.to_ledger_snapshot();
    snapshot.sequence_number += TTL + 1;
    snapshot.timestamp += 86400;
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
    });
}
