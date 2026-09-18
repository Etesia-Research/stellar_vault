use crate::{math, types::*};
use soroban_sdk::{contractclient, token::TokenClient, Address, Env};

#[contractclient(name = "ReflectorClient")]
pub trait Reflector {
    fn base(e: Env) -> FeedAsset;
    fn decimals(e: Env) -> u32;
    fn resolution(e: Env) -> u32;
    fn lastprice(e: Env, asset: FeedAsset) -> Option<PriceData>;
}
#[contractclient(name = "PairClient")]
pub trait Pair {
    fn token_0(e: Env) -> Address;
    fn token_1(e: Env) -> Address;
    fn get_reserves(e: Env) -> (i128, i128);
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

pub fn pool(e: &Env, usdc: &Address, s: &Source) -> Result<Observation, Error> {
    let c = PairClient::new(e, &s.pool);
    let t0 = c
        .try_token_0()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let t1 = c
        .try_token_1()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let (asset, quote) = if s.asset_is_token0 {
        (t0, t1)
    } else {
        (t1, t0)
    };
    if asset != s.asset || quote != *usdc {
        return Err(Error::Identity);
    }
    let ad = TokenClient::new(e, &asset)
        .try_decimals()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let qd = TokenClient::new(e, &quote)
        .try_decimals()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    if ad != s.decimals || qd != 7 {
        return Err(Error::Identity);
    }
    let (r0, r1) = c
        .try_get_reserves()
        .map_err(|_| Error::Invocation)?
        .map_err(|_| Error::Invocation)?;
    let (asset_reserve, usdc_reserve) = if s.asset_is_token0 {
        (r0, r1)
    } else {
        (r1, r0)
    };
    if asset_reserve < s.min_asset_reserve || usdc_reserve < s.min_usdc_reserve {
        return Err(Error::Depth);
    }
    Ok(Observation {
        price: math::ratio(e, usdc_reserve, 7, asset_reserve, s.decimals)?,
        timestamp: e.ledger().timestamp(),
        ledger: e.ledger().sequence(),
        asset_reserve,
        usdc_reserve,
    })
}
