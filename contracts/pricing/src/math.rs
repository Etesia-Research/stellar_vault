use crate::types::{Error, Observation};
use soroban_sdk::{Env, Vec, U256};

fn narrow(n: U256) -> Result<i128, Error> {
    let n = n.to_u128().ok_or(Error::Overflow)?;
    let n = i128::try_from(n).map_err(|_| Error::Overflow)?;
    if n <= 0 {
        return Err(Error::Invalid);
    }
    Ok(n)
}

/// Each operand is <=127 bits and each decimal scale <=100 bits; products fit U256.
/// A single final floor, including differently scaled conversion legs.
pub fn ratio(e: &Env, n: i128, nd: u32, d: i128, dd: u32) -> Result<i128, Error> {
    if n <= 0 || d <= 0 || nd > 18 || dd > 18 {
        return Err(Error::Invalid);
    }
    let exponent = 12 + dd as i32 - nd as i32;
    let mut n = U256::from_u128(e, n as u128);
    let mut d = U256::from_u128(e, d as u128);
    let scale = U256::from_u128(e, 10u128.pow(exponent.unsigned_abs()));
    if exponent >= 0 {
        n = n.mul(&scale);
    } else {
        d = d.mul(&scale);
    }
    narrow(n.div(&d))
}

pub fn twap(e: &Env, samples: &Vec<Observation>) -> Result<i128, Error> {
    if samples.len() != 3 {
        return Err(Error::Bootstrap);
    }
    let a = samples.get(0).unwrap();
    let b = samples.get(1).unwrap();
    let c = samples.get(2).unwrap();
    if a.price <= 0
        || b.price <= 0
        || c.price <= 0
        || a.timestamp >= b.timestamp
        || b.timestamp >= c.timestamp
    {
        return Err(Error::Invalid);
    }
    let p = |v: i128| U256::from_u128(e, v as u128);
    let dt = |v: u64| U256::from_u128(e, v as u128);
    let sum = p(a.price)
        .add(&p(b.price))
        .mul(&dt(b.timestamp - a.timestamp))
        .add(
            &p(b.price)
                .add(&p(c.price))
                .mul(&dt(c.timestamp - b.timestamp)),
        );
    narrow(sum.div(&dt(c.timestamp - a.timestamp).mul(&U256::from_u32(e, 2))))
}

pub fn within(e: &Env, price: i128, reference: i128, limit: u32) -> bool {
    // ceil(delta * 10000 / price) <= limit, without intermediate rounding.
    U256::from_u128(e, price.abs_diff(reference)).mul(&U256::from_u32(e, 10_000))
        <= U256::from_u128(e, price as u128).mul(&U256::from_u32(e, limit))
}
