use anchor_lang::prelude::*;
use crate::safe_number::safe_number::SafeNumber;

pub mod constant_curve;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum TradeDirection {
    AtoB = 0,
    BtoA = 1,
}

pub trait CurveCalculator {
    fn calculate_initial_price(&self) -> SafeNumber;
    fn calculate_amount_out(
        initial_price: SafeNumber,
        amount: u64,
        direction: TradeDirection
    ) -> u64;
}
