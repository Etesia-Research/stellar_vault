use crate::types::{Error, MAX_AMOUNT};
use soroban_sdk::{panic_with_error, Env, U256};

#[inline(never)]
pub fn nonnegative(e: &Env, n: i128) {
    if !(0..=MAX_AMOUNT).contains(&n) {
        panic_with_error!(e, Error::Invalid);
    }
}
#[inline(never)]
pub fn positive(e: &Env, n: i128) {
    nonnegative(e, n);
    if n == 0 {
        panic_with_error!(e, Error::Invalid);
    }
}
#[inline(never)]
pub fn add(e: &Env, a: i128, b: i128) -> i128 {
    a.checked_add(b)
        .unwrap_or_else(|| panic_with_error!(e, Error::Overflow))
}
#[inline(never)]
pub fn sub(e: &Env, a: i128, b: i128) -> i128 {
    a.checked_sub(b)
        .unwrap_or_else(|| panic_with_error!(e, Error::Overflow))
}
/// Nonnegative checked 256-bit multiply/divide. No saturating financial math.
#[inline(never)]
pub fn mul_div(e: &Env, a: i128, b: i128, d: i128, ceil: bool) -> i128 {
    if a < 0 || b < 0 || d <= 0 {
        panic_with_error!(e, Error::Invalid);
    }
    let product = U256::from_u128(e, a as u128).mul(&U256::from_u128(e, b as u128));
    let divisor = U256::from_u128(e, d as u128);
    let mut q = product.div(&divisor);
    if ceil && product.rem_euclid(&divisor) != U256::from_u32(e, 0) {
        q = q.add(&U256::from_u32(e, 1));
    }
    let n = q
        .to_u128()
        .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
    i128::try_from(n).unwrap_or_else(|_| panic_with_error!(e, Error::Overflow))
}

/// Fractional remainder of a*b/d, scaled without narrowing the product.
#[inline(never)]
pub fn fraction(e: &Env, a: i128, b: i128, d: i128, scale: i128) -> i128 {
    let rem = U256::from_u128(e, a as u128)
        .mul(&U256::from_u128(e, b as u128))
        .rem_euclid(&U256::from_u128(e, d as u128));
    rem.mul(&U256::from_u128(e, scale as u128))
        .div(&U256::from_u128(e, d as u128))
        .to_u128()
        .unwrap() as i128
}
