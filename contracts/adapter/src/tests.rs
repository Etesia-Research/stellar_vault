extern crate std;
use super::*;
use etesia_vault::{
    testutils::Fixture,
    types::{SEED_ASSETS, YEAR},
};
use soroban_sdk::{
    map,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    String,
};

#[allow(clippy::too_many_arguments)]
mod upstream {
    soroban_sdk::contractimport!(file = "../../vendor/defindex/target/wasm32-unknown-unknown/release/defindex_vault.optimized.wasm");
}
fn adapter(f: &Fixture, parent: &Address) -> Address {
    let args: Vec<Val> = vec![
        &f.e,
        f.vault.clone().into_val(&f.e),
        parent.clone().into_val(&f.e),
    ];
    f.e.register(Adapter, (&f.usdc, args))
}
#[test]
fn exact_parent_auth_uses_asset_units_and_isolates_parents() {
    let f = Fixture::new(false, false);
    let e = &f.e;
    let a = adapter(&f, &f.user);
    let c = AdapterClient::new(e, &a);
    let amount = 100_000_000;
    assert_eq!(c.asset(), f.usdc);
    assert_eq!(c.balance(&f.user), 0);
    assert!(c.try_balance(&f.admin).is_err());
    assert!(c.try_deposit(&amount, &f.admin).is_err());
    e.mock_auths(&[MockAuth {
        address: &f.user,
        invoke: &MockAuthInvoke {
            contract: &a,
            fn_name: "deposit",
            args: (amount, &f.user).into_val(e),
            sub_invokes: &[MockAuthInvoke {
                contract: &f.usdc,
                fn_name: "transfer",
                args: (&f.user, &a, amount).into_val(e),
                sub_invokes: &[],
            }],
        },
    }]);
    assert_eq!(c.deposit(&amount, &f.user), amount);
    assert_eq!(f.client().balance(&a), amount * 100_000);
    e.mock_auths(&[MockAuth {
        address: &f.user,
        invoke: &MockAuthInvoke {
            contract: &a,
            fn_name: "withdraw",
            args: (amount, &f.user, &f.user).into_val(e),
            sub_invokes: &[],
        },
    }]);
    assert_eq!(c.withdraw(&amount, &f.user, &f.user), 0);
}
#[test]
fn adapter_failures_and_pending_fee_balance() {
    let f = Fixture::new(false, false);
    let a = adapter(&f, &f.user);
    let c = AdapterClient::new(&f.e, &a);
    f.e.mock_all_auths();
    assert!(c.try_deposit(&0, &f.user).is_err());
    assert!(c.try_withdraw(&0, &f.user, &f.user).is_err());
    assert!(c.try_harvest(&f.user, &Some(Bytes::new(&f.e))).is_err());
    c.deposit(&SEED_ASSETS, &f.user);
    f.advance(YEAR, 10);
    let expected = f.client().convert_to_assets(&f.client().balance(&a));
    assert_eq!(c.balance(&f.user), expected);
    assert!(expected < SEED_ASSETS);
    assert!(c.try_withdraw(&1, &f.user, &a).is_err());
    assert!(c
        .try_withdraw(&(SEED_ASSETS * 10), &f.user, &f.user)
        .is_err());
    c.harvest(&f.user, &None);
    assert_eq!(c.balance(&f.user), expected);
    c.withdraw(&expected, &f.user, &f.user);
    assert!(c.balance(&f.user) <= 1);
}
#[test]
fn upstream_parent_invest_loss_fees_unwind_failure_and_recovery() {
    let f = Fixture::new(false, false);
    let e = &f.e;
    let parent = Address::generate(e);
    let a = adapter(&f, &parent);
    let assets = vec![
        e,
        upstream::AssetStrategySet {
            address: f.usdc.clone(),
            strategies: vec![
                e,
                upstream::Strategy {
                    address: a.clone(),
                    name: String::from_str(e, "Etesia"),
                    paused: false,
                },
            ],
        },
    ];
    let roles = map![
        e,
        (0u32, f.guardian.clone()),
        (1u32, f.recipient.clone()),
        (2u32, f.admin.clone()),
        (3u32, f.executor.clone())
    ];
    let metadata = map![
        e,
        (
            String::from_str(e, "name"),
            String::from_str(e, "D2 parent")
        ),
        (String::from_str(e, "symbol"), String::from_str(e, "D2"))
    ];
    e.register_at(
        &parent,
        upstream::WASM,
        (
            assets,
            roles,
            1000u32,
            f.recipient.clone(),
            500u32,
            f.router.clone(),
            metadata,
            false,
        ),
    );
    let c = upstream::Client::new(e, &parent);
    let strategy = AdapterClient::new(e, &a);
    e.mock_all_auths();
    let (_, _, _) = c.deposit(
        &vec![e, SEED_ASSETS],
        &vec![e, SEED_ASSETS],
        &f.user,
        &false,
    );
    let shares = c.balance(&f.user);
    c.rebalance(
        &f.executor,
        &vec![e, upstream::Instruction::Invest(a.clone(), SEED_ASSETS)],
    );
    assert_eq!(strategy.balance(&parent), SEED_ASSETS);
    f.donate(&f.usdc, SEED_ASSETS / 5);
    e.mock_all_auths();
    let appreciated = strategy.balance(&parent);
    assert!(appreciated > SEED_ASSETS);
    // The parent is actually invested. Draining local fixture liquidity must roll back its burn.
    let cash = f.client().liquid_assets();
    TokenClient::new(e, &f.usdc).burn(&f.vault, &cash);
    f.donate(&f.reserve, cash);
    e.mock_all_auths();
    let before = c.balance(&f.user);
    assert!(c.try_withdraw(&shares, &vec![e, 0], &f.user).is_err());
    assert_eq!(c.balance(&f.user), before);
    // Restore strategy liquidity through the actual permissionless vault entrypoint.
    // Start a fresh aggregate window; the donated gain is now part of its NAV baseline.
    f.advance(0, 17_280);
    e.mock_auths(&[]);
    let mut remaining = cash;
    let mut nonce = 0;
    while remaining > 0 {
        let leg = remaining.min(cash / 5);
        f.client().unwind(
            &vec![
                e,
                etesia_vault::types::Action::Swap(f.swap(&f.reserve, &f.usdc, leg)),
            ],
            &nonce,
            &u64::MAX,
        );
        remaining -= leg;
        nonce += 1;
    }
    e.mock_all_auths();
    let out = c.withdraw(&shares, &vec![e, 0], &f.user);
    assert!(out.get(0).unwrap() > SEED_ASSETS);
    assert_eq!(c.balance(&f.user), 0);
    assert!(strategy.balance(&parent) < SEED_ASSETS / 100);
}
