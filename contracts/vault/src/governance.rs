use crate::{
    fees, pricing,
    storage::{self, Key},
    types::*,
    Vault, VaultArgs, VaultClient,
};
use soroban_sdk::{contractimpl, panic_with_error, symbol_short, xdr::ToXdr, BytesN, Env};

#[contractimpl]
impl Vault {
    pub fn pause(e: Env) {
        storage::enter(&e);
        storage::config(&e).guardian.require_auth();
        let mut s = storage::state(&e);
        s.paused = true;
        storage::save(&e, &s);
        e.events().publish((symbol_short!("pause"), 1u32), true);
        storage::leave(&e);
    }
    pub fn announce(e: Env, change: Change) -> BytesN<32> {
        storage::enter(&e);
        storage::config(&e).admin.require_auth();
        if e.storage().instance().has(&Key::Pending) {
            panic_with_error!(&e, Error::Timelock);
        }
        if let Change::Fees(ref f) = change {
            fees::validate(&e, f);
        }
        let s = storage::state(&e);
        let hash = e
            .crypto()
            .sha256(
                &(
                    symbol_short!("config_v1"),
                    e.ledger().network_id(),
                    e.current_contract_address(),
                    s.version,
                    change.clone(),
                )
                    .to_xdr(&e),
            )
            .to_bytes();
        let earliest = e
            .ledger()
            .sequence()
            .checked_add(ROLE_DELAY)
            .unwrap_or_else(|| panic_with_error!(&e, Error::Overflow));
        let p = Pending {
            hash: hash.clone(),
            earliest,
            change,
        };
        e.storage().instance().set(&Key::Pending, &p);
        e.events().publish((symbol_short!("announce"), 1u32), p);
        storage::leave(&e);
        hash
    }
    pub fn cancel(e: Env, hash: BytesN<32>) {
        storage::enter(&e);
        storage::config(&e).guardian.require_auth();
        if storage::pending(&e).hash != hash {
            panic_with_error!(&e, Error::Timelock);
        }
        e.storage().instance().remove(&Key::Pending);
        e.events().publish((symbol_short!("cancel"), 1u32), hash);
        storage::leave(&e);
    }
    pub fn apply_change(e: Env, hash: BytesN<32>) {
        storage::enter(&e);
        let p = storage::pending(&e);
        if p.hash != hash || e.ledger().sequence() < p.earliest {
            panic_with_error!(&e, Error::Timelock);
        }
        let mut c = storage::config(&e);
        match p.change.clone() {
            Change::Fees(f) => {
                let a = pricing::positive_equity(&e);
                fees::settle(&e, Some(a), false);
                fees::validate(&e, &f);
                c.fees = f;
            }
            Change::Role(role, address) => {
                address.require_auth();
                if address == e.current_contract_address() {
                    panic_with_error!(&e, Error::Invalid);
                }
                match role {
                    Role::Admin => c.admin = address,
                    Role::Executor => c.executor = address,
                    Role::Guardian => c.guardian = address,
                }
            }
            Change::Resume => {
                pricing::positive_equity(&e);
                let mut s = storage::state(&e);
                s.paused = false;
                storage::save(&e, &s);
            }
        }
        e.storage().instance().set(&Key::Config, &c);
        let mut s = storage::state(&e);
        s.version = s
            .version
            .checked_add(1)
            .unwrap_or_else(|| panic_with_error!(&e, Error::Overflow));
        storage::save(&e, &s);
        e.storage().instance().remove(&Key::Pending);
        e.events()
            .publish((symbol_short!("config"), 1u32), (s.version, p));
        storage::leave(&e);
    }
    pub fn pending_change(e: Env) -> Option<Pending> {
        e.storage().instance().get(&Key::Pending)
    }
}
