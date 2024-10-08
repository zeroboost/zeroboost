use std::ops::{ Div, Mul, Sub };

use crate::safe_number::safe_number::{ Math, NewSafeNumber, SafeNumber };

use super::{ CurveCalculator, TradeDirection };

pub struct ConstantCurveCalculator {
    supply: f64,
    liquidity_percentage: f64,
    maximum_token_b_reserve_balance: f64,
}

impl ConstantCurveCalculator {
    pub fn new(
        supply: u64,
        liquidity_percentage: u8,
        maximum_token_b_reserve_balance: u64
    ) -> ConstantCurveCalculator {
        ConstantCurveCalculator {
            supply: supply as f64,
            liquidity_percentage: liquidity_percentage as f64,
            maximum_token_b_reserve_balance: maximum_token_b_reserve_balance as f64,
        }
    }

    pub fn get_token_b_reserve_balance(&self) -> f64 {
        self.maximum_token_b_reserve_balance.mul(self.liquidity_percentage).div(100_f64)
    }

    pub fn get_liquidity_supply(&self) -> f64 {
        self.supply.mul(self.liquidity_percentage).div(100_f64)
    }

    pub fn get_bounding_curve_supply(&self) -> f64 {
        self.supply.sub(self.get_liquidity_supply())
    }
}

impl CurveCalculator for ConstantCurveCalculator {
    fn calculate_initial_price(&self) -> SafeNumber {
        let supply = self.get_bounding_curve_supply();
        let token_b_reserve_balance = self.get_token_b_reserve_balance();

        f64::new(token_b_reserve_balance.div(supply))
    }

    fn calculate_amount_out(
        initial_price: SafeNumber,
        amount: u64,
        trade_direction: TradeDirection
    ) -> u64 {
        match trade_direction {
            TradeDirection::AtoB => initial_price.mul(amount.into()).unwrap(),
            TradeDirection::BtoA => initial_price.inverse_div(amount.into()).unwrap(),
        }
    }
}

#[cfg(test)]
mod constant_curve_test {
    use std::ops::Mul;

    use crate::curve::CurveCalculator;

    use super::ConstantCurveCalculator;

    #[test]
    pub fn sell_mint_fraction_to_meet_maximum_token_b_balance() {
        let supply = (1_000_000_000).mul((10_u64).pow(6));
        let liquidity_percentage = 50;
        let maximum_token_b_reserve_balance = (13656).mul((10_u64).pow(7));

        let curve = ConstantCurveCalculator::new(
            supply,
            liquidity_percentage,
            maximum_token_b_reserve_balance
        );

        let supply = curve.get_bounding_curve_supply() as u64;
        let maximum_token_b_reserve_balance = curve.get_token_b_reserve_balance() as u64;

        let initial_price = curve.calculate_initial_price();

        let token_amount_out = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            maximum_token_b_reserve_balance,
            crate::curve::TradeDirection::BtoA
        );

        let token_b_amount_out = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            supply,
            crate::curve::TradeDirection::AtoB
        );

        assert_eq!(
            initial_price.unwrap::<f64>(),
            0.00013656,
            "assert valid initial price with correct percision"
        );
        assert_eq!(
            token_amount_out,
            supply,
            "assert when bought total token equal to curve supply"
        );
        assert_eq!(
            token_b_amount_out,
            maximum_token_b_reserve_balance,
            "assert when sell equal to curve token B supply"
        );
    }
}
