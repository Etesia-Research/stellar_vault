extern crate std;
use crate::{testutils::Fixture, types::*};
use soroban_sdk::{
    symbol_short,
    testutils::{Events, MockAuth, MockAuthInvoke},
    token::TokenClient,
    vec, IntoVal, Symbol, TryFromVal,
};

#[test]
fn funded_seed_and_all_standard_round_trips() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let e = &f.e;
    assert_eq!(c.total_supply(), SEED_ASSETS * SHARE_MULTIPLIER);
    assert_eq!(c.balance(&f.vault), c.total_supply());
    assert_eq!(c.query_asset(), f.usdc);
    assert_eq!(c.decimals(), 12);
    let shares = f.deposit(123_456_789);
    assert_eq!(shares, c.preview_deposit(&123_456_789));
    e.mock_all_auths();
    assert_eq!(
        c.withdraw(&12_345_678, &f.user, &f.user, &f.user),
        c.preview_withdraw(&12_345_678)
    );
    let assets = c.preview_mint(&12_345);
    assert_eq!(c.mint(&12_345, &f.user, &f.user, &f.user), assets);
    let remaining = c.balance(&f.user);
    let expected = c.preview_redeem(&remaining);
    assert_eq!(c.redeem(&remaining, &f.user, &f.user, &f.user), expected);
    assert_eq!(c.balance(&f.user), 0);
    assert_eq!(c.total_supply(), c.balance(&f.vault));
    assert!(c.try_burn(&f.vault, &1).is_err());
    assert!(c.try_transfer(&f.vault, &f.user, &1).is_err());
    assert!(c.try_deposit(&0, &f.user, &f.user, &f.user).is_err());
    assert_eq!(c.max_withdraw(&f.vault), 0);
    assert_eq!(c.max_deposit(&f.vault), 0);
}
#[test]
fn exact_deposit_auth_tree_and_no_unauthorized_issuance() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let amount = 10_000_000;
    assert!(c.try_deposit(&amount, &f.user, &f.user, &f.user).is_err());
    f.e.mock_auths(&[MockAuth {
        address: &f.user,
        invoke: &MockAuthInvoke {
            contract: &f.vault,
            fn_name: "deposit",
            args: (amount, &f.user, &f.user, &f.user).into_val(&f.e),
            sub_invokes: &[MockAuthInvoke {
                contract: &f.usdc,
                fn_name: "transfer",
                args: (&f.user, &f.vault, amount).into_val(&f.e),
                sub_invokes: &[],
            }],
        },
    }]);
    assert_eq!(
        c.deposit(&amount, &f.user, &f.user, &f.user),
        amount * SHARE_MULTIPLIER
    );
}
#[test]
fn pending_fee_preview_matches_cash_flows_and_fee_receiver_value() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.deposit(SEED_ASSETS);
    f.advance(YEAR, 100);
    let expected = c.preview_deposit(&SEED_ASSETS);
    let minted = f.deposit(SEED_ASSETS);
    assert_eq!(expected, minted);
    let fee_shares = c.balance(&f.recipient);
    let value = c.convert_to_assets(&fee_shares);
    assert!((value - 2 * SEED_ASSETS / 100).abs() <= 1);
    assert_eq!(c.state().last_fee, f.e.ledger().timestamp());
    let before = c.total_supply();
    c.collect_fees();
    assert_eq!(before, c.total_supply());
}
#[test]
fn gain_loss_recovery_and_performance_dilution() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.e.as_contract(&f.vault, || {
        let mut cfg = crate::storage::config(&f.e);
        cfg.fees.management_bps = 0;
        f.e.storage()
            .instance()
            .set(&crate::storage::Key::Config, &cfg);
    });
    f.donate(&f.usdc, SEED_ASSETS / 5);
    c.collect_fees();
    let fees = c.balance(&f.recipient);
    assert!((c.convert_to_assets(&fees) - SEED_ASSETS / 50).abs() <= 1);
    let high = c.state().high_water;
    f.e.mock_all_auths();
    TokenClient::new(&f.e, &f.usdc).burn(&f.vault, &(SEED_ASSETS / 5));
    f.e.mock_auths(&[]);
    c.collect_fees();
    assert_eq!(high, c.state().high_water);
    f.donate(&f.usdc, SEED_ASSETS / 5);
    c.collect_fees();
    assert_eq!(fees, c.balance(&f.recipient));
}
#[test]
fn long_dormancy_requires_permissionless_checkpoints() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.advance(3 * YEAR + 4, 10);
    assert!(c.try_preview_deposit(&10_000_000).is_err());
    for _ in 0..3 {
        c.checkpoint_fees();
    }
    assert_eq!(c.state().last_fee, 1_000_000 + 3 * YEAR);
    c.collect_fees();
    assert_eq!(c.state().last_fee, f.e.ledger().timestamp());
}
#[test]
fn transfers_allowances_expiration_and_delegated_exit() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let shares = f.deposit(SEED_ASSETS);
    f.e.mock_all_auths();
    c.approve(&f.user, &f.executor, &shares, &105);
    assert_eq!(c.allowance(&f.user, &f.executor), shares);
    c.transfer_from(&f.executor, &f.user, &f.admin, &1000);
    assert_eq!(c.balance(&f.admin), 1000);
    c.transfer(&f.admin, &f.user, &500);
    c.burn_from(&f.executor, &f.user, &500);
    f.advance(0, 6);
    assert_eq!(c.allowance(&f.user, &f.executor), 0);
    assert!(c
        .try_transfer_from(&f.executor, &f.user, &f.admin, &1)
        .is_err());
    c.approve(&f.user, &f.executor, &shares, &200);
    assert!(c.withdraw(&100, &f.user, &f.user, &f.executor) > 0);
    assert!(c.try_approve(&f.user, &f.executor, &1, &99).is_err());
    assert!(c.try_transfer(&f.user, &f.vault, &1).is_err());
}
#[test]
fn stale_price_basket_waives_only_exiting_entitlement() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let shares = f.deposit(SEED_ASSETS);
    f.donate(&f.xlm, SEED_ASSETS / 5);
    f.mark(&f.xlm, PRICE_SCALE, false);
    f.advance(YEAR, 10);
    assert!(c.try_redeem(&shares, &f.user, &f.user, &f.user).is_err());
    let old = c.state().high_water;
    f.e.mock_all_auths();
    let minima = vec![&f.e, 0, 0, 0, 0];
    let paid = c.redeem_in_kind(&shares, &f.user, &f.user, &minima, &u64::MAX);
    assert!(f
        .e
        .events()
        .all()
        .iter()
        .any(
            |(_, topics, _)| Symbol::try_from_val(&f.e, &topics.get(0).unwrap()).ok()
                == Some(symbol_short!("waiver"))
        ));
    assert!(paid.iter().any(|n| n > 0));
    assert_eq!(c.balance(&f.user), 0);
    assert_eq!(c.state().high_water, old);
    assert!(c.balance(&f.recipient) > 0);
}
#[test]
fn outstanding_supply_blocks_basket_and_failed_exit_preserves_everything() {
    let f = Fixture::new(true, false);
    let c = f.client();
    let shares = f.deposit(SEED_ASSETS);
    f.positions(SEED_ASSETS, 0, 0);
    let state = c.state();
    let user = c.balance(&f.user);
    f.e.mock_all_auths();
    assert!(c
        .try_redeem_in_kind(
            &shares,
            &f.user,
            &f.user,
            &vec![&f.e, 0, 0, 0, 0],
            &u64::MAX
        )
        .is_err());
    assert_eq!(c.state(), state);
    assert_eq!(c.balance(&f.user), user);
    c.recover_supply(&f.usdc, &SEED_ASSETS);
    assert!(c.holdings().iter().all(|h| h.supply == 0));
    c.redeem_in_kind(
        &shares,
        &f.user,
        &f.user,
        &vec![&f.e, 0, 0, 0, 0],
        &u64::MAX,
    );
}
#[test]
fn borrowed_principal_interest_liquidation_and_debt_rounding() {
    let f = Fixture::new(true, true);
    let c = f.client();
    let initial = c.total_assets();
    f.positions(0, SEED_ASSETS / 2, SEED_ASSETS / 4);
    assert_eq!(c.total_assets(), initial + SEED_ASSETS / 4);
    f.e.as_contract(&f.pool, || {
        f.e.storage()
            .instance()
            .set(&(symbol_short!("drate"), f.xlm.clone()), &(PRICE_SCALE + 1))
    });
    assert_eq!(c.total_assets(), initial + SEED_ASSETS / 4 - 1);
    f.positions(0, 0, SEED_ASSETS / 4);
    assert_eq!(c.total_assets(), initial - SEED_ASSETS / 4 - 1);
    assert_eq!(c.max_withdraw(&f.user), 0);
}
#[test]
fn target_auth_guarded_swap_and_nonce_replay() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let action = Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40));
    let p = f.plan(vec![&f.e, action], 0);
    assert!(c.try_execute(&p).is_err());
    f.auth(&f.executor, "execute", (p.clone(),).into_val(&f.e));
    c.execute(&p);
    assert_eq!(c.state().nonce, 1);
    assert_eq!(
        TokenClient::new(&f.e, &f.xlm).balance(&f.vault),
        SEED_ASSETS / 40
    );
    assert!(c.try_execute(&p).is_err());
}
#[test]
fn malicious_output_and_reentry_roll_back_nonce_fees_and_transfers() {
    for reentry in [false, true] {
        let f = Fixture::new(false, false);
        let c = f.client();
        let p = f.plan(
            vec![
                &f.e,
                Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40)),
            ],
            0,
        );
        f.e.as_contract(&f.router, || {
            f.e.storage()
                .instance()
                .set(&symbol_short!("output"), &1i128);
            f.e.storage()
                .instance()
                .set(&symbol_short!("reenter"), &reentry);
        });
        f.auth(&f.executor, "execute", (p.clone(),).into_val(&f.e));
        let state = c.state();
        let cash = c.liquid_assets();
        assert!(c.try_execute(&p).is_err());
        assert_eq!(state, c.state());
        assert_eq!(cash, c.liquid_assets());
    }
}
#[test]
fn pause_exits_and_timelocked_fee_role_handover() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.deposit(SEED_ASSETS);
    f.auth(&f.guardian, "pause", ().into_val(&f.e));
    c.pause();
    assert_eq!(c.max_deposit(&f.user), 0);
    let change = Change::Fees(Fees {
        management_bps: 0,
        performance_bps: 0,
        recipient: f.admin.clone(),
    });
    f.auth(&f.admin, "announce", (change.clone(),).into_val(&f.e));
    let hash = c.announce(&change);
    assert!(c.try_apply_change(&hash).is_err());
    f.advance(10, ROLE_DELAY);
    c.apply_change(&hash);
    assert_eq!(c.config().fees.management_bps, 0);
    assert!(c.pending_change().is_none());
    f.e.mock_all_auths();
    assert!(c.withdraw(&100, &f.user, &f.user, &f.user) > 0);
    let change = Change::Role(Role::Executor, f.user.clone());
    let hash = c.announce(&change);
    f.advance(0, ROLE_DELAY);
    f.e.mock_auths(&[]);
    assert!(c.try_apply_change(&hash).is_err());
    f.auth(&f.user, "apply_change", (hash.clone(),).into_val(&f.e));
    c.apply_change(&hash);
    assert_eq!(c.config().executor, f.user);
}
#[test]
fn generated_cash_flow_sequences_conserve_assets_and_supply() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let mut seed = 0x455445534941u64;
    let before = c.total_assets() + TokenClient::new(&f.e, &f.usdc).balance(&f.user);
    for _ in 0..100 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let n = i128::from(seed % 1_000_000 + 1);
        f.e.mock_all_auths();
        if seed % 2 == 0 || c.balance(&f.user) < n * SHARE_MULTIPLIER {
            c.deposit(&n, &f.user, &f.user, &f.user);
        } else {
            c.withdraw(&n, &f.user, &f.user, &f.user);
        }
        assert_eq!(
            c.total_assets() + TokenClient::new(&f.e, &f.usdc).balance(&f.user),
            before
        );
        assert_eq!(
            c.total_supply(),
            c.balance(&f.vault) + c.balance(&f.user) + c.balance(&f.recipient)
        );
    }
}

#[test]
fn python_rust_target_hash_golden_vector() {
    use soroban_sdk::{Address, BytesN};
    let f = Fixture::new(false, false);
    let e = &f.e;
    let t = Target {
        network: BytesN::from_array(e, &[1; 32]),
        vault: Address::from_str(
            e,
            "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHK3M",
        ),
        epoch: 7,
        expiry: 1789740000,
        model: BytesN::from_array(e, &[2; 32]),
        dataset: BytesN::from_array(e, &[3; 32]),
        holdings: BytesN::from_array(e, &[4; 32]),
        assets: vec![
            e,
            Address::from_str(
                e,
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM",
            ),
            Address::from_str(
                e,
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4",
            ),
        ],
        weights: vec![e, 250, 9750],
        yield_bps: 0,
        equity: 10_000_000_000,
        supply: 1_000_000_000_000_000,
    };
    assert_eq!(
        f.client().target_hash(&t).to_array(),
        [
            0x45, 0x58, 0x07, 0x3d, 0x15, 0x86, 0x19, 0xc1, 0x6b, 0x00, 0xc0, 0x3b, 0x27, 0xc2,
            0xb9, 0x0d, 0x60, 0xd7, 0xf7, 0x6a, 0x5c, 0x75, 0xe4, 0x39, 0x2a, 0xf6, 0x31, 0xe5,
            0xf8, 0x2b, 0x55, 0x64
        ]
    );
}
#[test]
fn yield_supply_cap_shrink_and_frozen_recovery_preserve_claims() {
    let f = Fixture::new(true, false);
    let c = f.client();
    let e = &f.e;
    let p = f.plan(
        vec![e, Action::Supply(f.usdc.clone(), SEED_ASSETS * 95 / 1000)],
        1000,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
    assert_eq!(c.total_assets(), SEED_ASSETS);
    assert_eq!(c.liquid_assets(), SEED_ASSETS * 905 / 1000);
    e.as_contract(&f.pool, || {
        e.storage().instance().set(&symbol_short!("frozen"), &true)
    });
    let before = c.state();
    assert!(c
        .try_recover_supply(&f.usdc, &(SEED_ASSETS * 95 / 1000))
        .is_err());
    assert_eq!(c.state(), before);
    e.as_contract(&f.pool, || {
        e.storage().instance().set(&symbol_short!("frozen"), &false)
    });
    c.recover_supply(&f.usdc, &(SEED_ASSETS * 95 / 1000));
    assert_eq!(c.liquid_assets(), SEED_ASSETS);
    f.advance(0, 1);
    let mut t = f.target(1000);
    t.epoch = 2;
    f.auth(&f.executor, "publish_target", (t.clone(),).into_val(e));
    let hash = c.publish_target(&t);
    let p = Plan {
        actions: vec![e, Action::Supply(f.usdc.clone(), SEED_ASSETS / 10)],
        target: hash,
        epoch: 2,
        nonce: 1,
        version: 1,
        deadline: t.expiry,
    };
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    assert_eq!(c.try_execute(&p), Err(Ok(Error::Limit.into())));
    assert_eq!(c.liquid_assets(), SEED_ASSETS);
}
#[test]
fn synthetic_borrow_swap_repay_release_preserves_net_equity() {
    let f = Fixture::new(true, true);
    let c = f.client();
    let e = &f.e;
    let amount = SEED_ASSETS / 20;
    // A single bounded plan both reduces target error and builds a small, healthy short.
    let p = f.plan(
        vec![
            e,
            Action::Collateral(f.usdc.clone(), amount * 3),
            Action::Borrow(f.xlm.clone(), amount),
            Action::Swap(f.swap(&f.xlm, &f.usdc, amount)),
            Action::Swap(f.swap(&f.usdc, &f.reserve, amount * 2)),
        ],
        0,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
    assert_eq!(c.total_assets(), SEED_ASSETS);
    assert_eq!(c.balance(&f.recipient), 0);
    assert!(c.holdings().iter().any(|h| h.debt == amount));
    // The keeper is absent; no executor authorization is installed for the public unwind.
    e.mock_auths(&[]);
    c.unwind(
        &vec![
            e,
            Action::Swap(f.swap(&f.usdc, &f.xlm, amount)),
            Action::Repay(f.xlm.clone(), amount),
            Action::Release(f.usdc.clone(), amount * 3),
        ],
        &1,
        &u64::MAX,
    );
    assert_eq!(c.total_assets(), SEED_ASSETS);
    assert!(c
        .holdings()
        .iter()
        .all(|h| h.debt == 0 && h.collateral == 0));
}
#[test]
fn flagship_cannot_borrow_and_paused_vault_allows_safe_repayment() {
    let f = Fixture::new(true, false);
    let c = f.client();
    let p = f.plan(vec![&f.e, Action::Borrow(f.xlm.clone(), 100)], 0);
    f.auth(&f.executor, "execute", (p.clone(),).into_val(&f.e));
    assert_eq!(c.try_execute(&p), Err(Ok(Error::BorrowingDisabled.into())));
    let f = Fixture::new(true, true);
    let c = f.client();
    f.positions(0, SEED_ASSETS, 1000);
    f.donate(&f.xlm, 1000);
    f.mark(&f.xlm, PRICE_SCALE, false);
    f.auth(&f.guardian, "pause", ().into_val(&f.e));
    c.pause();
    f.e.mock_auths(&[]);
    c.repay(&f.xlm, &1000);
    assert!(c.holdings().iter().all(|h| h.debt == 0));
}
#[test]
fn priced_public_unwind_restores_usdc_without_keeper() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.donate(&f.reserve, SEED_ASSETS / 10);
    let old = c.liquid_assets();
    c.unwind(
        &vec![
            &f.e,
            Action::Swap(f.swap(&f.reserve, &f.usdc, SEED_ASSETS / 10)),
        ],
        &0,
        &u64::MAX,
    );
    assert_eq!(c.liquid_assets(), old + SEED_ASSETS / 10);
    assert_eq!(c.state().nonce, 1);
    assert_eq!(
        c.try_unwind(&vec![&f.e, Action::Borrow(f.xlm.clone(), 1)], &1, &0),
        Err(Ok(Error::Expired.into()))
    );
    assert_eq!(
        c.try_unwind(&vec![&f.e, Action::Borrow(f.xlm.clone(), 1)], &0, &u64::MAX),
        Err(Ok(Error::Replay.into()))
    );
}
#[test]
fn reject_expired_changed_or_unbounded_plans_before_custody_changes() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let p = f.plan(
        vec![
            &f.e,
            Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40)),
        ],
        0,
    );
    for case in 0..5 {
        let mut bad = p.clone();
        let expected = match case {
            0 => {
                bad.deadline = 0;
                Error::Expired
            }
            1 => {
                bad.version = 2;
                Error::Replay
            }
            2 => {
                bad.epoch = 2;
                Error::Target
            }
            3 => {
                bad.actions = vec![&f.e];
                Error::Limit
            }
            _ => {
                bad.nonce = 1;
                Error::Replay
            }
        };
        f.auth(&f.executor, "execute", (bad.clone(),).into_val(&f.e));
        assert_eq!(c.try_execute(&bad), Err(Ok(expected.into())));
        assert_eq!(c.state().nonce, 0);
        assert_eq!(c.liquid_assets(), SEED_ASSETS);
    }
    let shares = f.deposit(SEED_ASSETS / 10);
    assert!(shares > 0);
    f.auth(&f.executor, "execute", (p.clone(),).into_val(&f.e));
    assert_eq!(c.try_execute(&p), Err(Ok(Error::Target.into())));
}
#[test]
fn price_validity_divergence_insolvency_and_liquidity_views() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.deposit(SEED_ASSETS);
    f.donate(&f.xlm, 1000);
    for mark in [
        Mark {
            price: 0,
            reference: PRICE_SCALE,
            timestamp: f.e.ledger().timestamp(),
            ready: true,
        },
        Mark {
            price: PRICE_SCALE * 2,
            reference: PRICE_SCALE,
            timestamp: f.e.ledger().timestamp(),
            ready: true,
        },
        Mark {
            price: PRICE_SCALE,
            reference: PRICE_SCALE,
            timestamp: 1,
            ready: true,
        },
        Mark {
            price: PRICE_SCALE,
            reference: PRICE_SCALE,
            timestamp: u64::MAX,
            ready: true,
        },
    ] {
        f.e.as_contract(&f.oracle, || f.e.storage().instance().set(&f.xlm, &mark));
        assert_eq!(c.try_total_assets(), Err(Ok(Error::Pricing.into())));
        assert_eq!(c.max_withdraw(&f.user), 0);
    }
    f.mark(&f.xlm, PRICE_SCALE, true);
    assert!(c.max_withdraw(&f.user) > 0);
    assert!(c.max_redeem(&f.user) > 0);
    assert!(c.max_mint(&f.user) > 0);
    assert_eq!(
        c.name(),
        soroban_sdk::String::from_str(&f.e, "Etesia Vault")
    );
    assert_eq!(c.symbol(), soroban_sdk::String::from_str(&f.e, "ETESIA"));
    let f = Fixture::new(true, true);
    f.positions(0, 0, SEED_ASSETS * 2);
    assert!(f.client().total_assets() < 0);
    assert_eq!(
        f.client().try_preview_deposit(&1),
        Err(Ok(Error::Insolvent.into()))
    );
}
#[test]
fn bounded_user_methods_reject_bad_quotes_and_expired_calls() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.deposit(SEED_ASSETS);
    f.e.mock_all_auths();
    let state = c.state();
    assert_eq!(
        c.try_deposit_bounded(&1, &f.user, &f.user, &f.user, &MAX_AMOUNT, &u64::MAX),
        Err(Ok(Error::Slippage.into()))
    );
    assert_eq!(
        c.try_mint_bounded(&100_000, &f.user, &f.user, &f.user, &0, &u64::MAX),
        Err(Ok(Error::Slippage.into()))
    );
    assert_eq!(
        c.try_withdraw_bounded(&1, &f.user, &f.user, &f.user, &0, &u64::MAX),
        Err(Ok(Error::Slippage.into()))
    );
    assert_eq!(
        c.try_redeem_bounded(&100_000, &f.user, &f.user, &f.user, &100, &u64::MAX),
        Err(Ok(Error::Slippage.into()))
    );
    assert_eq!(
        c.try_deposit_bounded(&1, &f.user, &f.user, &f.user, &1, &0),
        Err(Ok(Error::Expired.into()))
    );
    assert_eq!(state, c.state());
    c.extend_ttl(&vec![&f.e, f.user.clone()], &vec![&f.e, f.executor.clone()]);
    assert_eq!(c.try_convert_to_shares(&-1), Err(Ok(Error::Invalid.into())));
    assert_eq!(c.try_convert_to_assets(&-1), Err(Ok(Error::Invalid.into())));
}
#[test]
fn guardian_cancel_and_mature_resume() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.e.mock_all_auths();
    c.pause();
    let hash = c.announce(&Change::Resume);
    c.cancel(&hash);
    assert!(c.pending_change().is_none());
    assert_eq!(c.try_apply_change(&hash), Err(Ok(Error::Timelock.into())));
    let hash = c.announce(&Change::Resume);
    f.advance(0, ROLE_DELAY);
    c.apply_change(&hash);
    assert!(!c.state().paused);
    let invalid = Change::Fees(Fees {
        management_bps: 201,
        performance_bps: 1000,
        recipient: f.recipient.clone(),
    });
    assert_eq!(c.try_announce(&invalid), Err(Ok(Error::Invalid.into())));
}

#[test]
fn material_rewards_block_priced_flows_until_claimed_and_held_rewards_count_once() {
    use crate::blend::{ReserveEmissionData, UserEmissionData};
    let f = Fixture::new(true, false);
    let c = f.client();
    let e = &f.e;
    f.positions(SEED_ASSETS / 10, 0, 0);
    e.as_contract(&f.pool, || {
        e.storage().instance().set(
            &(symbol_short!("emission"), 1u32),
            &ReserveEmissionData {
                expiration: e.ledger().timestamp() + 100,
                eps: 100_000_000,
                index: 1_000_000_000_000_000,
                last_time: e.ledger().timestamp() - 1,
            },
        );
        e.storage().persistent().set(
            &(f.vault.clone(), 1u32),
            &UserEmissionData {
                index: 0,
                accrued: 1,
            },
        );
    });
    assert_eq!(c.try_total_assets(), Err(Ok(Error::Rewards.into())));
    assert!(c
        .try_redeem_in_kind(&1, &f.user, &f.user, &vec![e, 0, 0, 0, 0], &u64::MAX)
        .is_err());
    // A zero-claim entitlement uses the same checked projection, without recognizing forecast yield.
    e.as_contract(&f.pool, || {
        e.storage().instance().set(
            &(symbol_short!("emission"), 1u32),
            &ReserveEmissionData {
                expiration: e.ledger().timestamp(),
                eps: 0,
                index: 0,
                last_time: e.ledger().timestamp(),
            },
        );
        e.storage().persistent().set(
            &(f.vault.clone(), 1u32),
            &UserEmissionData {
                index: 0,
                accrued: 0,
            },
        );
    });
    assert_eq!(c.total_assets(), SEED_ASSETS * 11 / 10);
    e.as_contract(&f.vault, || {
        let mut cfg = crate::storage::config(e);
        let i = cfg.assets.iter().position(|a| a.address == f.risk).unwrap() as u32;
        let mut a = cfg.assets.get(i).unwrap();
        a.kind = AssetKind::Reward;
        cfg.assets.set(i, a);
        e.storage()
            .instance()
            .set(&crate::storage::Key::Config, &cfg);
    });
    assert_eq!(c.claim_rewards(), 0);
    f.donate(&f.risk, 123);
    assert_eq!(c.total_assets(), SEED_ASSETS * 11 / 10 + 123);
}
#[test]
fn unsafe_routes_and_caller_directed_recipients_are_rejected() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let e = &f.e;
    let p = f.plan(
        vec![e, Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40))],
        0,
    );
    for case in 0..7 {
        let mut s = f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40);
        let expected = match case {
            0 => {
                s.auth = vec![
                    e,
                    RouteAuth::Transfer(f.usdc.clone(), f.executor.clone(), s.amount),
                ];
                Error::Unauthorized
            }
            1 => {
                s.auth = vec![
                    e,
                    RouteAuth::Transfer(f.risk.clone(), f.router.clone(), s.amount),
                ];
                Error::Unauthorized
            }
            2 => {
                s.minimum = 1;
                Error::Slippage
            }
            3 => {
                let mut d = s.distribution.get(0).unwrap();
                d.parts = 0;
                s.distribution.set(0, d);
                Error::Invalid
            }
            4 => {
                let mut d = s.distribution.get(0).unwrap();
                d.path = vec![e, f.xlm.clone(), f.usdc.clone()];
                s.distribution.set(0, d);
                Error::Invalid
            }
            5 => {
                let mut d = s.distribution.get(0).unwrap();
                d.protocol_id = Protocol::Aqua;
                s.distribution.set(0, d);
                Error::Invalid
            }
            _ => {
                s.auth = vec![
                    e,
                    RouteAuth::Invoke(RouteInvocation {
                        contract: f.router.clone(),
                        function: symbol_short!("drain"),
                        args: vec![e],
                        children: vec![e],
                    }),
                ];
                Error::Unauthorized
            }
        };
        let mut bad = p.clone();
        bad.actions = vec![e, Action::Swap(s)];
        f.auth(&f.executor, "execute", (bad.clone(),).into_val(e));
        assert_eq!(c.try_execute(&bad), Err(Ok(expected.into())));
        assert_eq!(c.state().nonce, 0);
    }
}
#[test]
fn repeated_individually_legal_public_unwinds_obey_aggregate_turnover() {
    let f = Fixture::new(false, false);
    let c = f.client();
    let e = &f.e;
    f.donate(&f.reserve, SEED_ASSETS);
    let amount = SEED_ASSETS / 5;
    for nonce in 0..5 {
        c.unwind(
            &vec![e, Action::Swap(f.swap(&f.reserve, &f.usdc, amount))],
            &nonce,
            &u64::MAX,
        );
    }
    f.donate(&f.reserve, amount);
    let state = c.state();
    assert_eq!(
        c.try_unwind(
            &vec![e, Action::Swap(f.swap(&f.reserve, &f.usdc, amount))],
            &5,
            &u64::MAX
        ),
        Err(Ok(Error::Limit.into()))
    );
    assert_eq!(state, c.state());
    f.advance(0, 17_280);
    c.unwind(
        &vec![e, Action::Swap(f.swap(&f.reserve, &f.usdc, amount))],
        &5,
        &u64::MAX,
    );
    assert_eq!(c.state().nonce, 6);
}
#[test]
fn nonprofitable_fee_settlement_never_resets_mark_and_fractional_fees_accumulate() {
    let f = Fixture::new(false, false);
    let e = &f.e;
    e.as_contract(&f.vault, || {
        let mut s = crate::storage::state(e);
        let mut fees = crate::storage::config(e).fees;
        let h = s.high_water;
        assert_eq!(crate::fees::performance(e, &mut s, &fees, 0), 0);
        assert_eq!(s.high_water, h);
        fees.performance_bps = 0;
        assert_eq!(
            crate::fees::performance(e, &mut s, &fees, SEED_ASSETS * 2),
            0
        );
        assert!(s.high_water > h);
        let mut tiny = s.clone();
        tiny.supply = 1;
        tiny.high_water = 0;
        fees.performance_bps = 1000;
        assert_eq!(crate::fees::performance(e, &mut tiny, &fees, 1), 0);
        assert_eq!(tiny.high_water, 0);
        assert!(crate::math::mul_div(e, i128::MAX, i128::MAX, i128::MAX, false) == i128::MAX);
    });
    for _ in 0..100 {
        f.advance(1, 0);
        f.client().checkpoint_fees();
    }
    assert!(f.client().balance(&f.recipient) > 0);
    assert_eq!(f.client().state().last_fee, 1_000_100);
}
#[test]
fn health_guard_rejects_stale_pool_oracle_and_undercollateralized_positions() {
    let f = Fixture::new(true, true);
    let c = f.client();
    let e = &f.e;
    f.positions(0, SEED_ASSETS / 10, SEED_ASSETS / 10);
    f.donate(&f.reserve, SEED_ASSETS / 20);
    assert_eq!(
        c.try_unwind(
            &vec![
                e,
                Action::Swap(f.swap(&f.reserve, &f.usdc, SEED_ASSETS / 20))
            ],
            &0,
            &u64::MAX
        ),
        Err(Ok(Error::Limit.into()))
    );
    f.positions(0, SEED_ASSETS / 2, SEED_ASSETS / 10);
    f.mark(&f.xlm, PRICE_SCALE, false);
    assert_eq!(
        c.try_unwind(
            &vec![
                e,
                Action::Swap(f.swap(&f.reserve, &f.usdc, SEED_ASSETS / 20))
            ],
            &0,
            &u64::MAX
        ),
        Err(Ok(Error::Pricing.into()))
    );
}

#[test]
fn shrinking_residual_blocks_allocations_until_pool_recovery_and_allows_reentry() {
    let f = Fixture::new(true, false);
    let c = f.client();
    let e = &f.e;
    let p = f.plan(
        vec![e, Action::Supply(f.usdc.clone(), SEED_ASSETS * 95 / 1000)],
        1000,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
    f.advance(0, 1);
    let mut t = f.target(1000);
    t.epoch = 2;
    // Trend grows to 40%: residual is 55%, with 5.5% equity allowed in lending.
    for (i, a) in c.config().assets.iter().enumerate() {
        t.weights.set(
            i as u32,
            match a.kind {
                AssetKind::Settlement => 800,
                AssetKind::Xlm => 250,
                AssetKind::Risk => 4000,
                AssetKind::Reserve => 4950,
                _ => 0,
            },
        );
    }
    f.auth(&f.executor, "publish_target", (t.clone(),).into_val(e));
    let hash = c.publish_target(&t);
    let mut p = Plan {
        target: hash,
        epoch: 2,
        nonce: 1,
        version: 1,
        deadline: t.expiry,
        actions: vec![e, Action::Swap(f.swap(&f.usdc, &f.risk, SEED_ASSETS / 20))],
    };
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    assert_eq!(c.try_execute(&p), Err(Ok(Error::Limit.into())));
    p.actions = vec![e, Action::Withdraw(f.usdc.clone(), SEED_ASSETS * 4 / 100)];
    e.as_contract(&f.pool, || {
        e.storage().instance().set(&symbol_short!("frozen"), &true)
    });
    let state = c.state();
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    assert!(c.try_execute(&p).is_err());
    assert_eq!(c.state(), state);
    e.as_contract(&f.pool, || {
        e.storage().instance().set(&symbol_short!("frozen"), &false)
    });
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
    assert_eq!(
        c.holdings()
            .iter()
            .find(|h| h.asset == f.usdc)
            .unwrap()
            .supply,
        SEED_ASSETS * 55 / 1000
    );
    // Anyone can complete recovery without the executor, then a fresh target permits re-entry.
    e.mock_auths(&[]);
    c.recover_supply(&f.usdc, &(SEED_ASSETS * 55 / 1000));
    f.advance(0, 1);
    t = f.target(1000);
    t.epoch = 3;
    f.auth(&f.executor, "publish_target", (t.clone(),).into_val(e));
    p.target = c.publish_target(&t);
    p.epoch = 3;
    p.nonce = 2;
    p.actions = vec![e, Action::Supply(f.usdc.clone(), SEED_ASSETS / 20)];
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
}

#[test]
fn prefunded_donations_cannot_rebootstrap_or_steal_rounding_from_bounded_deposits() {
    use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address};
    let f = Fixture::new(false, false);
    let e = &f.e;
    let address = Address::generate(e);
    e.mock_all_auths();
    StellarAssetClient::new(e, &f.usdc).mint(&address, &SEED_ASSETS);
    e.register_at(
        &address,
        crate::Vault,
        (f.client().config(), f.admin.clone()),
    );
    let c = crate::VaultClient::new(e, &address);
    assert_eq!(c.total_supply(), SEED_ASSETS * SHARE_MULTIPLIER);
    assert_eq!(c.total_assets(), SEED_ASSETS * 2);
    let before = c.state();
    assert!(c
        .try_deposit_bounded(&1, &f.user, &f.user, &f.user, &SHARE_MULTIPLIER, &u64::MAX)
        .is_err());
    assert_eq!(before, c.state());
    let shares = c.deposit(&SEED_ASSETS, &f.user, &f.user, &f.user);
    let mark = c.state().high_water;
    c.redeem(&shares, &f.user, &f.user, &f.user);
    assert!(c.state().high_water >= mark);
    assert_eq!(c.balance(&address), SEED_ASSETS * SHARE_MULTIPLIER);
    assert!(c.try_transfer(&address, &f.user, &1).is_err());
    assert!(c.try_burn(&address, &1).is_err());
}

#[test]
fn management_collection_frequency_has_the_documented_prorated_bound() {
    let yearly = Fixture::new(false, false);
    let daily = Fixture::new(false, false);
    yearly.advance(YEAR, 1);
    yearly.client().collect_fees();
    for _ in 0..365 {
        daily.advance(YEAR / 365, 1);
        daily.client().collect_fees();
    }
    let y =
        yearly.client().balance(&yearly.recipient) as f64 / yearly.client().total_supply() as f64;
    let d = daily.client().balance(&daily.recipient) as f64 / daily.client().total_supply() as f64;
    assert!((y - 0.01).abs() < 1e-12);
    assert!((d - (1. - (1_f64 - 0.01 / 365.).powi(365))).abs() < 1e-12);
    assert!(y - d < 0.00005); // under 0.5 basis points of unchanged NAV per year
}

#[test]
fn ttl_extension_and_restored_snapshot_preserve_supply_nonce_mark_and_pending_change() {
    use crate::storage::{Key, TTL_EXTEND};
    use soroban_sdk::testutils::{storage::Instance, storage::Persistent};
    let f = Fixture::new(false, false);
    let e = &f.e;
    let c = f.client();
    f.deposit(SEED_ASSETS);
    let p = f.plan(
        vec![e, Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40))],
        0,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    c.execute(&p);
    e.mock_all_auths();
    c.approve(&f.user, &f.executor, &123, &999999);
    c.announce(&Change::Resume);
    c.extend_ttl(&vec![e, f.user.clone()], &vec![e, f.executor.clone()]);
    e.as_contract(&f.vault, || {
        assert!(e.storage().instance().get_ttl() >= TTL_EXTEND - 1);
        assert!(
            e.storage()
                .persistent()
                .get_ttl(&Key::Balance(f.user.clone()))
                >= TTL_EXTEND - 1
        );
    });
    let state = c.state();
    let snapshot = e.to_ledger_snapshot();
    let mut restored = snapshot.clone();
    restored.sequence_number += 4_000_000;
    // Model restoration by retaining archived entry payloads and renewing their TTL only.
    for (_, (_, ttl)) in restored.ledger_entries.iter_mut() {
        if ttl.is_some() {
            *ttl = Some(restored.sequence_number + TTL_EXTEND);
        }
    }
    for ((key, (data, _)), (new_key, (new_data, _))) in snapshot
        .ledger_entries
        .iter()
        .zip(restored.ledger_entries.iter())
    {
        assert_eq!(key, new_key);
        assert_eq!(data, new_data);
    }
    let r = soroban_sdk::Env::from_ledger_snapshot(restored);
    use std::string::ToString;
    let vault = soroban_sdk::Address::from_str(&r, &f.vault.to_string().to_string());
    let user = soroban_sdk::Address::from_str(&r, &f.user.to_string().to_string());
    let executor = soroban_sdk::Address::from_str(&r, &f.executor.to_string().to_string());
    r.as_contract(&vault, || {
        assert_eq!(crate::storage::state(&r), state);
        assert!(crate::storage::balance(&r, &user) > 0);
        assert_eq!(crate::storage::allowance(&r, &user, &executor), 0); // original expiry is preserved
        assert!(r.storage().instance().has(&Key::Pending));
        assert!(r.storage().instance().has(&Key::Target));
    });
}

#[test]
fn reward_claim_measures_custody_and_rejects_false_report_atomically() {
    let f = Fixture::new(true, false);
    let e = &f.e;
    let c = f.client();
    e.as_contract(&f.vault, || {
        let mut config = crate::storage::config(e);
        let i = config
            .assets
            .iter()
            .position(|a| a.address == f.risk)
            .unwrap() as u32;
        let mut asset = config.assets.get(i).unwrap();
        asset.kind = AssetKind::Reward;
        config.assets.set(i, asset);
        e.storage()
            .instance()
            .set(&crate::storage::Key::Config, &config);
    });
    e.as_contract(&f.pool, || {
        e.storage()
            .instance()
            .set(&symbol_short!("claim"), &123i128);
        e.storage()
            .instance()
            .set(&symbol_short!("reward"), &f.risk);
        e.storage()
            .instance()
            .set(&symbol_short!("badclaim"), &1i128);
    });
    assert_eq!(c.try_claim_rewards(), Err(Ok(Error::Rewards.into())));
    assert_eq!(TokenClient::new(e, &f.risk).balance(&f.vault), 0);
    e.as_contract(&f.pool, || {
        e.storage()
            .instance()
            .set(&symbol_short!("badclaim"), &0i128)
    });
    assert_eq!(c.claim_rewards(), 123);
    assert_eq!(c.total_assets(), SEED_ASSETS + 123);
    assert_eq!(c.claim_rewards(), 0);
    assert_eq!(c.total_assets(), SEED_ASSETS + 123);
}

#[test]
fn public_unwind_can_reduce_an_overweight_position_in_bounded_steps() {
    let f = Fixture::new(false, false);
    f.donate(&f.risk, SEED_ASSETS * 2);
    let amount = SEED_ASSETS * 3 / 5;
    f.client().unwind(
        &vec![&f.e, Action::Swap(f.swap(&f.risk, &f.usdc, amount))],
        &0,
        &u64::MAX,
    );
    assert_eq!(f.client().total_assets(), SEED_ASSETS * 3);
    assert_eq!(
        TokenClient::new(&f.e, &f.risk).balance(&f.vault),
        SEED_ASSETS * 14 / 10
    );
    assert_eq!(f.client().state().nonce, 1);
}

#[test]
fn cash_flows_adjust_budget_capital_without_resetting_consumed_turnover() {
    let f = Fixture::new(false, false);
    let shares = f.deposit(SEED_ASSETS);
    let c = f.client();
    assert_eq!(c.state().window_equity, SEED_ASSETS * 2);
    f.donate(&f.risk, SEED_ASSETS / 10);
    c.unwind(
        &vec![
            &f.e,
            Action::Swap(f.swap(&f.risk, &f.usdc, SEED_ASSETS / 10)),
        ],
        &0,
        &u64::MAX,
    );
    let before = c.state();
    f.e.mock_all_auths();
    let out = c.redeem(&(shares / 2), &f.user, &f.user, &f.user);
    assert_eq!(c.state().window_equity, before.window_equity - out);
    assert_eq!(c.state().turnover, before.turnover);
    assert_eq!(c.state().nonce, before.nonce);
}

#[test]
fn multihop_auth_preserves_existing_intermediate_holdings_and_supports_split_distributions() {
    use crate::testutils::HopFixture;
    use soroban_sdk::token::StellarAssetClient;
    let f = Fixture::new(false, false);
    let e = &f.e;
    let hop = e.register(HopFixture, ());
    e.mock_all_auths();
    for asset in [&f.risk, &f.xlm] {
        StellarAssetClient::new(e, asset).mint(&hop, &SEED_ASSETS);
    }
    f.donate(&f.risk, 100);
    e.as_contract(&f.vault, || {
        let mut config = crate::storage::config(e);
        config.route_contracts.push_back(hop.clone());
        e.storage()
            .instance()
            .set(&crate::storage::Key::Config, &config);
    });
    e.as_contract(&f.router, || {
        e.storage().instance().set(&symbol_short!("hop"), &hop)
    });
    let mut swap = f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 40);
    swap.distribution = vec![e];
    for protocol in [
        Protocol::Soroswap,
        Protocol::Phoenix,
        Protocol::Aqua,
        Protocol::Comet,
    ] {
        swap.distribution.push_back(DexDistribution {
            protocol_id: protocol,
            path: vec![e, f.usdc.clone(), f.risk.clone(), f.xlm.clone()],
            parts: 1,
            bytes: Some(vec![
                e,
                soroban_sdk::BytesN::from_array(e, &[1; 32]),
                soroban_sdk::BytesN::from_array(e, &[2; 32]),
            ]),
        });
    }
    swap.auth = vec![
        e,
        RouteAuth::Invoke(RouteInvocation {
            contract: hop.clone(),
            function: symbol_short!("swap"),
            args: (&f.vault, &f.usdc, &f.risk, &f.xlm, swap.amount).into_val(e),
            children: vec![
                e,
                RouteAuth::Transfer(f.usdc.clone(), hop.clone(), swap.amount),
                RouteAuth::Transfer(f.risk.clone(), hop.clone(), swap.amount),
            ],
        }),
    ];
    let p = f.plan(vec![e, Action::Swap(swap)], 0);
    // Taking even one pre-existing intermediate atom invalidates the whole route.
    e.as_contract(&hop, || {
        e.storage().instance().set(&symbol_short!("short"), &1i128)
    });
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    assert_eq!(f.client().try_execute(&p), Err(Ok(Error::Slippage.into())));
    assert_eq!(TokenClient::new(e, &f.risk).balance(&f.vault), 100);
    e.as_contract(&hop, || {
        e.storage().instance().set(&symbol_short!("short"), &0i128)
    });
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    f.client().execute(&p);
    assert_eq!(TokenClient::new(e, &f.risk).balance(&f.vault), 100);
}

#[test]
fn extreme_inputs_and_fee_intermediates_fail_atomically_instead_of_wrapping() {
    let f = Fixture::new(false, false);
    let c = f.client();
    f.e.mock_all_auths();
    let before = c.state();
    assert_eq!(
        c.try_deposit(&i128::MAX, &f.user, &f.user, &f.user),
        Err(Ok(Error::Invalid.into()))
    );
    assert_eq!(before, c.state());
    f.donate(&f.usdc, 1_000_000_000_000_000_000_000);
    let cash = c.liquid_assets();
    assert_eq!(c.try_collect_fees(), Err(Ok(Error::Overflow.into())));
    assert_eq!(before, c.state());
    assert_eq!(cash, c.liquid_assets());
}

#[test]
fn recovery_rejects_excess_debt_purchases_and_unrepaid_tokens_while_paused() {
    for case in 0..5 {
        let f = Fixture::new(true, true);
        let e = &f.e;
        let c = f.client();
        let amount = SEED_ASSETS / 20;
        f.positions(0, amount * 3, if case == 0 { 1 } else { amount });
        f.donate(&f.reserve, amount * 2);
        if case == 2 {
            f.donate(&f.xlm, amount / 2);
        }
        f.auth(&f.guardian, "pause", ().into_val(e));
        c.pause();
        e.mock_auths(&[]);
        let state = c.state();
        let holdings = c.holdings();
        // Increased cash must not disguise oversized or unconsumed purchases,
        // including cases where repayment tokens are already held.
        let mut actions = vec![
            e,
            Action::Swap(f.swap(&f.reserve, &f.usdc, amount * 2)),
            Action::Swap(f.swap(&f.usdc, &f.xlm, amount)),
        ];
        if case == 3 {
            actions.push_back(Action::Repay(f.xlm.clone(), amount / 2));
        }
        if case == 4 {
            e.as_contract(&f.router, || {
                e.storage()
                    .instance()
                    .set(&symbol_short!("output"), &(amount * 2));
            });
        }
        assert_eq!(
            c.try_unwind(&actions, &0, &u64::MAX),
            Err(Ok(Error::Limit.into()))
        );
        assert_eq!(c.state(), state);
        assert_eq!(c.holdings(), holdings);
    }
}

#[test]
fn recovery_consumes_only_the_missing_repayment_assets() {
    let f = Fixture::new(true, true);
    let e = &f.e;
    let c = f.client();
    let amount = SEED_ASSETS / 20;
    f.positions(0, amount * 3, amount);
    f.donate(&f.xlm, amount / 2);
    c.unwind(
        &vec![
            e,
            Action::Swap(f.swap(&f.usdc, &f.xlm, amount / 2)),
            Action::Repay(f.xlm.clone(), amount),
        ],
        &0,
        &u64::MAX,
    );
    let xlm = c.holdings().iter().find(|h| h.asset == f.xlm).unwrap();
    assert_eq!((xlm.spot, xlm.debt), (0, 0));
}

#[test]
fn recovery_allows_partial_deleveraging_while_limits_remain_breached() {
    let f = Fixture::new(true, true);
    let e = &f.e;
    let c = f.client();
    f.positions(0, SEED_ASSETS, SEED_ASSETS);
    f.auth(&f.guardian, "pause", ().into_val(e));
    c.pause();
    e.mock_auths(&[]);
    let equity = c.total_assets();
    for nonce in 0..2 {
        c.unwind(
            &vec![
                e,
                Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 10)),
                Action::Repay(f.xlm.clone(), SEED_ASSETS / 10),
            ],
            &nonce,
            &u64::MAX,
        );
    }
    assert_eq!(c.total_assets(), equity);
    let debt = c.holdings().iter().find(|h| h.asset == f.xlm).unwrap().debt;
    assert_eq!(debt, SEED_ASSETS * 8 / 10);
    assert!(debt > equity / 2);
    assert_eq!(c.state().nonce, 2);
}

#[test]
fn recovery_rejects_deleveraging_that_worsens_pool_health() {
    let f = Fixture::new(true, true);
    let e = &f.e;
    let c = f.client();
    f.positions(0, SEED_ASSETS, SEED_ASSETS);
    let before = c.holdings();
    let state = c.state();
    assert_eq!(
        c.try_unwind(
            &vec![
                e,
                Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 10)),
                Action::Repay(f.xlm.clone(), SEED_ASSETS / 10),
                Action::Release(f.usdc.clone(), SEED_ASSETS / 5),
            ],
            &0,
            &u64::MAX,
        ),
        Err(Ok(Error::Limit.into()))
    );
    assert_eq!(c.holdings(), before);
    assert_eq!(c.state(), state);
}

#[test]
fn reward_assets_cannot_be_bought_but_can_be_realized() {
    let f = Fixture::new(false, false);
    let e = &f.e;
    let c = f.client();
    e.as_contract(&f.vault, || {
        let mut config = crate::storage::config(e);
        let i = config
            .assets
            .iter()
            .position(|a| a.address == f.risk)
            .unwrap() as u32;
        let mut asset = config.assets.get(i).unwrap();
        asset.kind = AssetKind::Reward;
        config.assets.set(i, asset);
        e.storage()
            .instance()
            .set(&crate::storage::Key::Config, &config);
    });
    let amount = SEED_ASSETS / 5;
    let p = f.plan(
        vec![
            e,
            Action::Swap(f.swap(&f.usdc, &f.reserve, amount)),
            Action::Swap(f.swap(&f.usdc, &f.risk, amount)),
        ],
        0,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    let state = c.state();
    let before = c.holdings();
    assert_eq!(c.try_execute(&p), Err(Ok(Error::Invalid.into())));
    assert_eq!(c.state(), state);
    assert_eq!(c.holdings(), before);
    f.donate(&f.risk, amount);
    c.unwind(
        &vec![e, Action::Swap(f.swap(&f.risk, &f.usdc, amount))],
        &0,
        &u64::MAX,
    );
    assert_eq!(TokenClient::new(e, &f.risk).balance(&f.vault), 0);
}

#[test]
fn recovery_rejects_less_debt_when_losses_increase_leverage() {
    let f = Fixture::new(true, true);
    let e = &f.e;
    let c = f.client();
    f.positions(0, SEED_ASSETS, SEED_ASSETS);
    f.donate(&f.reserve, SEED_ASSETS / 10);
    f.donate(&f.xlm, SEED_ASSETS / 10000);
    e.as_contract(&f.router, || {
        e.storage()
            .instance()
            .set(&symbol_short!("output"), &(SEED_ASSETS * 99 / 1000));
    });
    let before = c.holdings();
    assert_eq!(
        c.try_unwind(
            &vec![
                e,
                Action::Swap(f.swap(&f.reserve, &f.usdc, SEED_ASSETS / 10)),
                Action::Repay(f.xlm.clone(), SEED_ASSETS / 10000),
            ],
            &0,
            &u64::MAX,
        ),
        Err(Ok(Error::Limit.into()))
    );
    assert_eq!(c.holdings(), before);
    assert_eq!(c.state().nonce, 0);
}

#[test]
fn ordinary_execution_cannot_use_the_partial_deleveraging_exception() {
    let f = Fixture::new(true, true);
    let e = &f.e;
    let c = f.client();
    f.positions(0, SEED_ASSETS, SEED_ASSETS);
    let p = f.plan(
        vec![
            e,
            Action::Swap(f.swap(&f.usdc, &f.xlm, SEED_ASSETS / 10)),
            Action::Repay(f.xlm.clone(), SEED_ASSETS / 10),
        ],
        0,
    );
    f.auth(&f.executor, "execute", (p.clone(),).into_val(e));
    let before = c.holdings();
    assert_eq!(c.try_execute(&p), Err(Ok(Error::Limit.into())));
    assert_eq!(c.holdings(), before);
    assert_eq!(c.state().nonce, 0);
}
