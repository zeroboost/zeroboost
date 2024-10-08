use std::ops::Mul;

use curve::safe_number::safe_number::{ SafeNumber, NewSafeNumber };
use pyth_sdk_solana::Price;

pub fn get_estimated_raydium_cp_pool_creation_fee() -> u64 {
    (2).mul((10_u64).pow(6)) + (15).mul((10_u64).pow(8)) + (203938).mul((10_u64).pow(1))
}

pub fn price_to_number(price: Price) -> SafeNumber {
    f64::new((price.price as f64) / (10f64).powi(-price.expo as i32))
}
