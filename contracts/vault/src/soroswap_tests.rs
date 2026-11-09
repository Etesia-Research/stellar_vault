//! Native tests against mainnet Soroswap code: the Aggregator v3 (SHA-256 `437653…`), router
//! (`4c3db3eb…`), factory (`5db738b0…`) and pair (`18051456…`) WASMs, all verified by
//! `fixtures/wasm/SHA256SUMS`.
//!
//! `Env::default()` enforces the mainnet per-invocation resource limits after every top-level
//! call. Its budget also caps the host's shadow budget, which in these tests meters test-only
//! work (recording mode parses each WASM again on every call) and passes 40 MB on one swap, so
//! the budget's CPU cap is raised to the mainnet invocation limit (400M) and its memory cap to
//! 1 GiB; the mainnet per-invocation limits still apply.
extern crate std;
use crate::{execution::AggregatorClient, testutils::Fixture, types::*};
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Events},
    token::{StellarAssetClient, TokenClient},
    vec,
    xdr::{ContractEventBody, ScAddress, ScError, ScErrorCode, ScErrorType, ScVal},
    Address, BytesN, Env, IntoVal, TryFromVal, Val, Vec,
};

#[allow(clippy::too_many_arguments)]
mod v3 {
    soroban_sdk::contractimport!(file = "../../fixtures/wasm/soroswap_aggregator_v3.wasm");
}
#[allow(clippy::too_many_arguments)]
mod router {
    soroban_sdk::contractimport!(file = "../../fixtures/wasm/soroswap_router.wasm");
}
mod factory {
    soroban_sdk::contractimport!(file = "../../fixtures/wasm/soroswap_factory.wasm");
}
mod pair {
    soroban_sdk::contractimport!(file = "../../fixtures/wasm/soroswap_pair.wasm");
}

const PROTOCOL_BPS: u32 = 10;
const DEPTH: i128 = SEED_ASSETS * 100;
const AMOUNT: i128 = SEED_ASSETS / 40;
const FEE: i128 = AMOUNT * PROTOCOL_BPS as i128 / BPS;

// v3 with the test router and an Aqua placeholder, and three pairs seeded at parity with the
// oracle fixture: route_contracts = [USDC/XLM, XLM/RESERVE, USDC/RESERVE].
fn fixture() -> Fixture {
    let f = Fixture::with_venue(Env::default(), false, false, |e, admin, tokens| {
        let factory = e.register(factory::WASM, ());
        let pair_hash = e.deployer().upload_contract_wasm(pair::WASM);
        factory::Client::new(e, &factory).initialize(admin, &pair_hash);
        let router = e.register(router::WASM, ());
        let router_client = router::Client::new(e, &router);
        router_client.initialize(&factory);
        let treasury = Address::generate(e);
        let aqua_router = Address::generate(e);
        let v3 = e.register(
            v3::WASM,
            (
                admin,
                &treasury,
                PROTOCOL_BPS,
                PROTOCOL_BPS,
                &router,
                &aqua_router,
            ),
        );
        let lp = Address::generate(e);
        let (usdc, xlm, reserve) = (&tokens[0], &tokens[1], &tokens[2]);
        let mut pairs = Vec::new(e);
        for (a, b) in [(usdc, xlm), (xlm, reserve), (usdc, reserve)] {
            for t in [a, b] {
                StellarAssetClient::new(e, t).mint(&lp, &DEPTH);
            }
            router_client.add_liquidity(a, b, &DEPTH, &DEPTH, &0, &0, &lp, &u64::MAX);
            pairs.push_back(factory::Client::new(e, &factory).get_pair(a, b));
        }
        (v3, pairs)
    });
    f.e.cost_estimate()
        .budget()
        .reset_limits(400_000_000, 1 << 30);
    f
}
fn pair(f: &Fixture, i: u32) -> Address {
    f.client().config().route_contracts.get(i).unwrap()
}
fn hop(pair: &Address, token_in: &Address, token_out: &Address) -> Hop {
    Hop::Soroswap(SoroswapHop {
        pair: pair.clone(),
        token_in: token_in.clone(),
        token_out: token_out.clone(),
    })
}
fn route(f: &Fixture, amount_in: i128, hops: &[Hop]) -> Route {
    Route {
        amount_in,
        hops: Vec::from_slice(&f.e, hops),
    }
}
// A USDC -> XLM swap of AMOUNT over `routes`.
fn swap(f: &Fixture, routes: &[Route]) -> Swap {
    let mut s = f.swap(&f.usdc, &f.xlm, AMOUNT);
    s.routes = Vec::from_slice(&f.e, routes);
    s
}
// One route of one hop carrying the post-fee amount.
fn single(f: &Fixture, h: Hop) -> Swap {
    swap(f, &[route(f, AMOUNT - FEE, &[h])])
}
fn execute(f: &Fixture, s: Swap) -> Result<(), soroban_sdk::Error> {
    let p = f.plan(vec![&f.e, Action::Swap(s)], 0);
    f.auth(&f.executor, "execute", (p.clone(),).into_val(&f.e));
    match f.client().try_execute(&p) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.unwrap()),
    }
}
// The contract that raised the first error of the last call, and that error. v3 codes 1-12
// mean different things in the vault, so a failure is identified by its contract.
fn origin(e: &Env) -> (Address, ScError) {
    let error = ScVal::try_from_val(e, &symbol_short!("error").to_val()).unwrap();
    e.host()
        .get_diagnostic_events()
        .unwrap()
        .0
        .into_iter()
        .find_map(|h| {
            let ContractEventBody::V0(body) = &h.event.body;
            match (
                &h.event.contract_id,
                body.topics.first(),
                body.topics.get(1),
            ) {
                (Some(id), Some(topic), Some(ScVal::Error(err))) if *topic == error => Some((
                    Address::try_from_val(e, &ScAddress::Contract(id.clone())).unwrap(),
                    err.clone(),
                )),
                _ => None,
            }
        })
        .unwrap()
}
// How a host auth failure below a native contract reaches that contract's caller.
fn context_error() -> soroban_sdk::Error {
    soroban_sdk::Error::from_type_and_code(ScErrorType::Context, ScErrorCode::InvalidAction)
}
fn sc(e: &Env, v: Val) -> ScVal {
    ScVal::try_from_val(e, &v).unwrap()
}

#[test]
fn single_route_pays_the_protocol_fee_and_v3_pulls_exactly_amount_in() {
    let f = fixture();
    let e = &f.e;
    let (usdc, xlm) = (TokenClient::new(e, &f.usdc), TokenClient::new(e, &f.xlm));
    let treasury = v3::Client::new(e, &f.router).config().treasury;
    let soroswap = v3::Client::new(e, &f.router).config().soroswap_router;
    // The routes carry the post-fee amount; the vault still spends and checks the gross amount.
    let expected = router::Client::new(e, &soroswap)
        .router_get_amounts_out(&(AMOUNT - FEE), &vec![e, f.usdc.clone(), f.xlm.clone()])
        .get(1)
        .unwrap();
    let s = single(&f, hop(&f.pair, &f.usdc, &f.xlm));
    assert!(expected >= s.minimum);
    let before = usdc.balance(&f.vault);
    assert_eq!(execute(&f, s), Ok(()));
    let transfer = sc(e, symbol_short!("transfer").to_val());
    let from_vault: std::vec::Vec<_> = e
        .events()
        .all()
        .filter_by_contract(&f.usdc)
        .events()
        .iter()
        .filter_map(|event| {
            let ContractEventBody::V0(body) = &event.body;
            (body.topics.first() == Some(&transfer)
                && body.topics.get(1) == Some(&sc(e, f.vault.to_val())))
            .then(|| (body.topics[2].clone(), body.data.clone()))
        })
        .collect();
    assert_eq!(
        from_vault,
        [(sc(e, f.router.to_val()), sc(e, AMOUNT.into_val(e)))]
    );
    assert_eq!(before - usdc.balance(&f.vault), AMOUNT);
    assert_eq!(usdc.balance(&treasury), FEE);
    assert_eq!(xlm.balance(&f.vault), expected);
}
#[test]
fn multi_route_swap_through_two_paths() {
    let f = fixture();
    let (xlm_reserve, usdc_reserve) = (pair(&f, 1), pair(&f, 2));
    let half = (AMOUNT - FEE) / 2;
    let s = swap(
        &f,
        &[
            route(&f, half, &[hop(&f.pair, &f.usdc, &f.xlm)]),
            route(
                &f,
                AMOUNT - FEE - half,
                &[
                    hop(&usdc_reserve, &f.usdc, &f.reserve),
                    hop(&xlm_reserve, &f.reserve, &f.xlm),
                ],
            ),
        ],
    );
    let minimum = s.minimum;
    assert_eq!(execute(&f, s), Ok(()));
    let xlm = TokenClient::new(&f.e, &f.xlm).balance(&f.vault);
    assert!(xlm >= minimum && xlm < AMOUNT);
    // v3 forwards the intermediate RESERVE; the vault never holds it.
    assert_eq!(TokenClient::new(&f.e, &f.reserve).balance(&f.vault), 0);
}
#[test]
fn bad_sum_and_broken_chain_fail_bad_route_in_v3() {
    let bad_route = ScError::Contract(5);
    for case in 0..2 {
        let f = fixture();
        let routes = if case == 0 {
            // The gross amount: the shares must add up to amount_in - fee.
            [route(&f, AMOUNT, &[hop(&f.pair, &f.usdc, &f.xlm)])]
        } else {
            // RISK does not follow RESERVE.
            [route(
                &f,
                AMOUNT - FEE,
                &[
                    hop(&pair(&f, 2), &f.usdc, &f.reserve),
                    hop(&pair(&f, 1), &f.risk, &f.xlm),
                ],
            )]
        };
        // The vault's caller sees code 5, which is also the vault's Expired.
        assert_eq!(
            execute(&f, swap(&f, &routes)),
            Err(soroban_sdk::Error::from_contract_error(5))
        );
        assert_eq!(origin(&f.e), (f.router.clone(), bad_route.clone()));
    }
}
#[test]
fn foreign_pair_and_aqua_hop_fail_unauthorized_in_the_vault() {
    for case in 0..2 {
        let f = fixture();
        let e = &f.e;
        let h = if case == 0 {
            hop(&Address::generate(e), &f.usdc, &f.xlm)
        } else {
            Hop::Aqua(AquaHop {
                steps: vec![
                    e,
                    (
                        vec![e, f.usdc.clone(), f.xlm.clone()],
                        BytesN::from_array(e, &[0; 32]),
                        f.xlm.clone(),
                    ),
                ],
                token_in: f.usdc.clone(),
            })
        };
        let s = single(&f, h);
        assert_eq!(execute(&f, s), Err(Error::Unauthorized.into()));
        assert_eq!(origin(e), (f.vault.clone(), ScError::Contract(3)));
    }
}
#[test]
fn mismatched_pair_fails_host_auth_at_the_input_token() {
    // XLM/RESERVE is in route_contracts but is not the USDC/XLM pool. v3 authorizes the
    // router's pull into the hop's `pair`; the router transfers into the real pair, so the
    // token rejects v3's authorization. The vault's caller sees a host Context error.
    let f = fixture();
    let s = single(&f, hop(&pair(&f, 1), &f.usdc, &f.xlm));
    assert_eq!(execute(&f, s), Err(context_error()));
    assert_eq!(
        origin(&f.e),
        (f.usdc.clone(), ScError::Auth(ScErrorCode::InvalidAction))
    );
}

// The vault's v3 call with the token-pull authorization made optional.
#[contract]
struct Caller;
#[contractimpl]
impl Caller {
    pub fn swap(e: Env, v3: Address, s: Swap, authorize: bool) -> i128 {
        let this = e.current_contract_address();
        if authorize {
            e.authorize_as_current_contract(vec![
                &e,
                InvokerContractAuthEntry::Contract(SubContractInvocation {
                    context: ContractContext {
                        contract: s.token_in.clone(),
                        fn_name: symbol_short!("transfer"),
                        args: (&this, &v3, s.amount).into_val(&e),
                    },
                    sub_invocations: vec![&e],
                }),
            ]);
        }
        AggregatorClient::new(&e, &v3).swap_exact_in(
            &this,
            &s.token_in,
            &s.amount,
            &s.token_out,
            &s.minimum,
            &s.routes,
            &None,
        )
    }
}
#[test]
fn without_the_invoker_auth_entry_the_pull_fails_host_auth() {
    let f = fixture();
    let e = &f.e;
    let caller = e.register(Caller, ());
    e.mock_all_auths();
    StellarAssetClient::new(e, &f.usdc).mint(&caller, &AMOUNT);
    e.mock_auths(&[]);
    let s = single(&f, hop(&f.pair, &f.usdc, &f.xlm));
    let c = CallerClient::new(e, &caller);
    assert_eq!(c.try_swap(&f.router, &s, &false), Err(Ok(context_error())));
    assert_eq!(
        origin(e),
        (f.usdc.clone(), ScError::Auth(ScErrorCode::InvalidAction))
    );
    assert!(c.swap(&f.router, &s, &true) >= s.minimum);
    assert_eq!(TokenClient::new(e, &f.usdc).balance(&caller), 0);
}
