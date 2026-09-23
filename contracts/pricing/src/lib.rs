#![no_std]
mod math;
pub mod sources;
pub mod types;
use soroban_sdk::{contract, contractimpl, xdr::ToXdr, Address, BytesN, Env, Vec};
use types::*;

#[contract]
pub struct Pricing;

fn config(e: &Env) -> Result<Config, Error> {
    let c = e
        .storage()
        .instance()
        .get(&Key::Config)
        .ok_or(Error::Config)?;
    e.storage().instance().extend_ttl(TTL / 2, TTL);
    Ok(c)
}
fn source(c: &Config, asset: &Address) -> Result<Source, Error> {
    c.sources
        .iter()
        .find(|s| s.asset == *asset)
        .ok_or(Error::Unknown)
}
fn evaluate(e: &Env, d: &mut Diagnostic) -> Result<(), Error> {
    let s = &d.source;
    let n = sources::feed(e, &s.numerator)?;
    d.numerator.push_back(n.clone());
    let (denominator, decimals, timestamp) = if let Some(f) = s.denominator.first() {
        let p = sources::feed(e, &f)?;
        d.denominator.push_back(p.clone());
        (p.price, f.decimals, n.timestamp.min(p.timestamp))
    } else {
        (1, 0, n.timestamp)
    };
    d.mark = Mark {
        price: math::ratio(e, n.price, s.numerator.decimals, denominator, decimals)?,
        timestamp,
    };
    Ok(())
}

#[contractimpl]
impl Pricing {
    pub fn __constructor(e: Env, c: Config) -> Result<(), Error> {
        if c.sources.is_empty() || c.sources.len() > 7 {
            return Err(Error::Config);
        }
        let mut seen = Vec::new(&e);
        for s in c.sources.iter() {
            if s.asset == c.usdc || seen.contains(s.asset.clone()) || s.denominator.len() > 1 {
                return Err(Error::Config);
            }
            seen.push_back(s.asset.clone());
            for f in [Some(s.numerator.clone()), s.denominator.first()]
                .into_iter()
                .flatten()
            {
                if f.decimals > 18 || f.max_age == 0 || f.max_age > 3600 || f.resolution == 0 {
                    return Err(Error::Config);
                }
            }
            if let Some(d) = s.denominator.first() {
                if d.base != s.numerator.base
                    || (d.asset != FeedAsset::Stellar(c.usdc.clone())
                        && d.asset != FeedAsset::Other(soroban_sdk::symbol_short!("USDC")))
                {
                    return Err(Error::Config);
                }
            } else if s.numerator.base != FeedAsset::Stellar(c.usdc.clone()) {
                return Err(Error::Config);
            }
        }
        e.storage().instance().set(&Key::Config, &c);
        e.storage().instance().extend_ttl(TTL / 2, TTL);
        Ok(())
    }
    pub fn config(e: Env) -> Result<Config, Error> {
        config(&e)
    }
    pub fn config_hash(e: Env) -> Result<BytesN<32>, Error> {
        Ok(e.crypto().sha256(&config(&e)?.to_xdr(&e)).into())
    }
    pub fn diagnose(e: Env, asset: Address) -> Result<Diagnostic, Error> {
        let c = config(&e)?;
        let s = source(&c, &asset)?;
        let mut d = Diagnostic {
            source: s,
            numerator: Vec::new(&e),
            denominator: Vec::new(&e),
            mark: Mark {
                price: 0,
                timestamp: 0,
            },
            error: 0,
        };
        d.error = evaluate(&e, &mut d)
            .err()
            .map(|err| err as u32)
            .unwrap_or(0);
        Ok(d)
    }
    pub fn mark(e: Env, asset: Address) -> Result<Mark, Error> {
        let c = config(&e)?;
        if asset == c.usdc {
            return Ok(Mark {
                price: SCALE,
                timestamp: e.ledger().timestamp(),
            });
        }
        let d = Self::diagnose(e, asset)?;
        if d.error != 0 {
            return Err(Error::try_from(soroban_sdk::Error::from_contract_error(d.error)).unwrap());
        }
        Ok(d.mark)
    }
}

#[cfg(test)]
mod tests;
