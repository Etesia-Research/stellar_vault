use crate::types::Error;
use soroban_sdk::{Env, U256};

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
