use anchor_lang::prelude::*;
use std::{ f64, ops::{ Div, Mul } };

pub const SAFE_NUMBER_SIZE: usize = 16 + 4;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct SafeNumber {
    pub value: u128,
    percision: i32,
}

pub trait Math {
    fn mul(&self, other: u128) -> SafeNumber;
    fn div(&self, other: u128) -> SafeNumber;
    fn inverse_div(&self, other: u128) -> SafeNumber;
}

pub trait Unwrap {
    fn unwrap(value: u128, percision: i32) -> Self;
}

pub trait NewSafeNumber {
    fn new(value: Self) -> SafeNumber;
}

impl Unwrap for u64 {
    fn unwrap(value: u128, percision: i32) -> Self {
        f64::unwrap(value, percision).round() as u64
    }
}

impl Unwrap for u128 {
    fn unwrap(value: u128, percision: i32) -> Self {
        f64::unwrap(value, percision).round() as u128
    }
}

impl Unwrap for f64 {
    fn unwrap(value: u128, percision: i32) -> Self {
        (value as f64).div((10_f64).powi(percision))
    }
}

impl NewSafeNumber for f64 {
    fn new(value: Self) -> SafeNumber {
        let percision = match value.to_string().split('.').nth(1) {
            Some(percision) => percision.len() as i32,
            None => 0,
        };

        SafeNumber {
            value: SafeNumber::wrap(value.into(), percision),
            percision,
        }
    }
}

impl NewSafeNumber for u64 {
    fn new(value: Self) -> SafeNumber {
        f64::new(value as f64)
    }
}

impl NewSafeNumber for u128 {
    fn new(value: Self) -> SafeNumber {
        f64::new(value as f64)
    }
}

impl SafeNumber {
    fn clone(value: u128, percision: i32) -> Self {
        Self { value, percision }
    }

    fn wrap(value: f64, percision: i32) -> u128 {
        value.mul((10_f64).powi(percision)) as u128
    }

    pub fn unwrap<U: Unwrap>(&self) -> U {
        U::unwrap(self.value, self.percision)
    }
}

impl Math for SafeNumber {
    fn mul(&self, other: u128) -> SafeNumber {
        Self::clone(self.value.mul(other), self.percision)
    }

    fn div(&self, other: u128) -> SafeNumber {
        Self::clone(self.value.div(other), self.percision)
    }

    fn inverse_div(&self, other: u128) -> SafeNumber {
        f64::new((other as f64).div(self.unwrap::<f64>()))
    }
}

impl PartialEq for SafeNumber {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.percision == other.percision
    }
}
