use crate::types::*;
use soroban_sdk::{contractclient, Env};

#[contractclient(name = "ReflectorClient")]
pub trait Reflector {
    fn base(e: Env) -> FeedAsset;
    fn decimals(e: Env) -> u32;
    fn resolution(e: Env) -> u32;
    fn lastprice(e: Env, asset: FeedAsset) -> Option<PriceData>;
}

pub fn age(now: u64, timestamp: u64, max_age: u64) -> Result<(), Error> {
    if timestamp > now {
        return Err(Error::Future);
    }
    if now - timestamp > max_age {
        return Err(Error::Stale);
    }
    Ok(())
}

pub fn feed(e: &Env, f: &Feed) -> Result<PriceData, Error> {
    let c = ReflectorClient::new(e, &f.contract);
    let base = c
        .try_base()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let decimals = c
        .try_decimals()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let resolution = c
        .try_resolution()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    if base != f.base || decimals != f.decimals || resolution != f.resolution {
        return Err(Error::Identity);
    }
    let p = c
        .try_lastprice(&f.asset)
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?
        .ok_or(Error::Missing)?;
    if p.price <= 0 {
        return Err(Error::Invalid);
    }
    age(e.ledger().timestamp(), p.timestamp, f.max_age)?;
    Ok(p)
}
