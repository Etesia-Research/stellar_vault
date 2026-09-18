#![no_std]
use defindex_strategy_core::{DeFindexStrategyTrait, StrategyError};
#[soroban_sdk::contractclient(name = "VaultClient")]
pub trait VaultInterface {
    fn query_asset(e: Env) -> Address;
    fn balance(e: Env, id: Address) -> i128;
    fn convert_to_assets(e: Env, shares: i128) -> i128;
    fn deposit(e: Env, assets: i128, receiver: Address, from: Address, operator: Address) -> i128;
    fn withdraw(e: Env, assets: i128, receiver: Address, owner: Address, operator: Address)
        -> i128;
    fn max_withdraw(e: Env, owner: Address) -> i128;
    fn collect_fees(e: Env) -> Val;
}
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, contracttype,
    token::TokenClient,
    vec, Address, Bytes, Env, IntoVal, Symbol, TryFromVal, Val, Vec,
};

#[contracttype]
#[derive(Clone)]
enum Key {
    Binding,
    Guard,
}
#[contracttype]
#[derive(Clone)]
struct Binding {
    asset: Address,
    vault: Address,
    parent: Address,
}
#[contract]
pub struct Adapter;
fn binding(e: &Env) -> Binding {
    e.storage().instance().get(&Key::Binding).unwrap()
}
fn enter(e: &Env, from: &Address) -> Result<Binding, StrategyError> {
    let b = binding(e);
    if *from != b.parent || e.storage().instance().get(&Key::Guard).unwrap_or(false) {
        return Err(StrategyError::NotAuthorized);
    }
    from.require_auth();
    e.storage().instance().set(&Key::Guard, &true);
    e.storage().instance().extend_ttl(172_800, 518_400);
    Ok(b)
}
fn leave(e: &Env) {
    e.storage().instance().set(&Key::Guard, &false);
}
fn balance(e: &Env, b: &Binding) -> i128 {
    let vault = VaultClient::new(e, &b.vault);
    vault.convert_to_assets(&vault.balance(&e.current_contract_address()))
}

#[contractimpl]
impl DeFindexStrategyTrait for Adapter {
    fn __constructor(e: Env, asset: Address, init_args: Vec<Val>) {
        assert!(init_args.len() == 2, "expected vault and parent");
        let vault = Address::try_from_val(&e, &init_args.get(0).unwrap()).unwrap();
        let parent = Address::try_from_val(&e, &init_args.get(1).unwrap()).unwrap();
        assert!(
            VaultClient::new(&e, &vault).query_asset() == asset,
            "underlying mismatch"
        );
        e.storage().instance().set(
            &Key::Binding,
            &Binding {
                asset,
                vault,
                parent,
            },
        );
        e.storage().instance().extend_ttl(172_800, 518_400);
    }
    fn asset(e: Env) -> Result<Address, StrategyError> {
        Ok(binding(&e).asset)
    }
    fn balance(e: Env, from: Address) -> Result<i128, StrategyError> {
        let b = binding(&e);
        if from != b.parent {
            return Err(StrategyError::NotAuthorized);
        }
        Ok(balance(&e, &b))
    }
    fn deposit(e: Env, amount: i128, from: Address) -> Result<i128, StrategyError> {
        if amount <= 0 {
            return Err(StrategyError::OnlyPositiveAmountAllowed);
        }
        let b = enter(&e, &from)?;
        let this = e.current_contract_address();
        TokenClient::new(&e, &b.asset).transfer(&from, &this, &amount);
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: b.asset.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (&this, &b.vault, amount).into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
        ]);
        VaultClient::new(&e, &b.vault).deposit(&amount, &this, &this, &this);
        let result = balance(&e, &b);
        leave(&e);
        Ok(result)
    }
    fn withdraw(e: Env, amount: i128, from: Address, to: Address) -> Result<i128, StrategyError> {
        if amount <= 0 {
            return Err(StrategyError::OnlyPositiveAmountAllowed);
        }
        let b = enter(&e, &from)?;
        if to == e.current_contract_address() || to == b.vault {
            return Err(StrategyError::NotAuthorized);
        }
        let vault = VaultClient::new(&e, &b.vault);
        let this = e.current_contract_address();
        if amount > vault.max_withdraw(&this) {
            return Err(StrategyError::InsufficientBalance);
        }
        let before = TokenClient::new(&e, &b.asset).balance(&to);
        vault.withdraw(&amount, &to, &this, &this);
        if TokenClient::new(&e, &b.asset).balance(&to) - before != amount {
            return Err(StrategyError::ExternalError);
        }
        let result = balance(&e, &b);
        leave(&e);
        Ok(result)
    }
    fn harvest(e: Env, from: Address, data: Option<Bytes>) -> Result<(), StrategyError> {
        if data.is_some() {
            return Err(StrategyError::InvalidArgument);
        }
        let b = enter(&e, &from)?;
        VaultClient::new(&e, &b.vault).collect_fees();
        leave(&e);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
