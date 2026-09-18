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
fn history(e: &Env, asset: &Address) -> History {
    let key = Key::History(asset.clone());
    let mut h: History = e.storage().temporary().get(&key).unwrap_or(History {
        completed: Vec::new(e),
        pending: Vec::new(e),
    });
    if let Some(p) = h.pending.first() {
        if p.ledger < e.ledger().sequence() {
            if h.completed.len() == 3 {
                h.completed.pop_front();
            }
            h.completed.push_back(p);
            h.pending = Vec::new(e);
        }
    }
    h
}
fn evaluate(e: &Env, d: &mut Diagnostic) -> Result<(), Error> {
    let s = &d.source;
    let n = sources::feed(e, &s.numerator)?;
    d.numerator.push_back(n.clone());
    let (denominator, decimals, mut oldest, mut newest) = if let Some(f) = s.denominator.first() {
        let p = sources::feed(e, &f)?;
        d.denominator.push_back(p.clone());
        (
            p.price,
            f.decimals,
            n.timestamp.min(p.timestamp),
            n.timestamp.max(p.timestamp),
        )
    } else {
        (1, 0, n.timestamp, n.timestamp)
    };
    d.mark.price = math::ratio(e, n.price, s.numerator.decimals, denominator, decimals)?;
    if d.samples.len() != 3 {
        return Err(Error::Bootstrap);
    }
    let mut previous: Option<Observation> = None;
    for p in d.samples.iter() {
        sources::age(e.ledger().timestamp(), p.timestamp, s.max_age)?;
        if p.ledger >= e.ledger().sequence() || p.price <= 0 {
            return Err(Error::Invalid);
        }
        if p.asset_reserve < s.min_asset_reserve || p.usdc_reserve < s.min_usdc_reserve {
            return Err(Error::Depth);
        }
        if let Some(prev) = previous {
            if p.ledger <= prev.ledger || p.timestamp <= prev.timestamp {
                return Err(Error::Spacing);
            }
            let gap = p.timestamp - prev.timestamp;
            if gap < s.min_spacing || gap > s.max_gap {
                return Err(Error::Spacing);
            }
        }
        oldest = oldest.min(p.timestamp);
        newest = newest.max(p.timestamp);
        previous = Some(p);
    }
    // The window is already unusable before an overdue observation resets it.
    sources::age(
        e.ledger().timestamp(),
        d.samples.last().unwrap().timestamp,
        s.max_gap,
    )?;
    if newest - oldest > s.max_skew {
        return Err(Error::Skew);
    }
    d.mark.reference = math::twap(e, &d.samples)?;
    d.mark.timestamp = oldest;
    if !math::within(e, d.mark.price, d.mark.reference, s.divergence_bps) {
        return Err(Error::Divergence);
    }
    d.mark.ready = true;
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
            if s.asset == c.usdc
                || seen.contains(s.asset.clone())
                || s.decimals > 18
                || s.denominator.len() > 1
                || s.min_asset_reserve <= 0
                || s.min_usdc_reserve <= 0
                || s.min_spacing == 0
                || s.min_spacing > s.max_gap
                || s.max_gap > s.max_age
                || s.max_age > 3600
                || s.min_spacing > s.max_age / 2
                || s.max_skew > 3600
                || s.max_skew < s.min_spacing * 2
                || s.divergence_bps > 500
            {
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
    pub fn upkeep(e: Env) -> Result<(), Error> {
        let c = config(&e)?;
        for s in c.sources.iter() {
            let key = Key::History(s.asset);
            if e.storage().temporary().has(&key) {
                e.storage().temporary().extend_ttl(&key, TTL / 2, TTL);
            }
        }
        Ok(())
    }
    pub fn observe(e: Env, asset: Address) -> Result<Observation, Error> {
        let c = config(&e)?;
        let s = source(&c, &asset)?;
        let mut h = history(&e, &asset);
        if !h.pending.is_empty() {
            return Err(Error::Spacing);
        }
        if let Some(last) = h.completed.last() {
            let now = e.ledger().timestamp();
            if now <= last.timestamp || now - last.timestamp < s.min_spacing {
                return Err(Error::Spacing);
            }
            // A broken collection interval cannot be filled with reconstructed history.
            if now - last.timestamp > s.max_gap {
                h.completed = Vec::new(&e);
            }
        }
        let p = sources::pool(&e, &c.usdc, &s)?;
        h.pending.push_back(p.clone());
        let key = Key::History(asset);
        e.storage().temporary().set(&key, &h);
        e.storage().temporary().extend_ttl(&key, TTL / 2, TTL);
        Ok(p)
    }
    pub fn diagnose(e: Env, asset: Address) -> Result<Diagnostic, Error> {
        let c = config(&e)?;
        let s = source(&c, &asset)?;
        let mut d = Diagnostic {
            source: s,
            numerator: Vec::new(&e),
            denominator: Vec::new(&e),
            samples: history(&e, &asset).completed,
            mark: Mark {
                price: 0,
                reference: 0,
                timestamp: 0,
                ready: false,
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
                reference: SCALE,
                timestamp: e.ledger().timestamp(),
                ready: true,
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
