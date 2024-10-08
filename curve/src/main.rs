use std::ops::Mul;

use curve::{ constant_curve::ConstantCurveCalculator, CurveCalculator, TradeDirection };

pub mod safe_number;
pub mod curve;

fn main() {
    let supply = (1_000_000_000).mul((10_u64).pow(6));
    let liquidity_percentage = 50;
    let maximum_token_b_reserve_balance = (13656).mul((10_u64).pow(7));

    let curve = ConstantCurveCalculator::new(
        supply,
        liquidity_percentage,
        maximum_token_b_reserve_balance
    );

    let supply = curve.get_bounding_curve_supply().round() as u64;
    let maximum_token_b_reserve_balance = curve.get_token_b_reserve_balance().round() as u64;
    let initial_price = curve.calculate_initial_price();

    println!("price={:?}", initial_price);
    println!("supply={}", supply);
    println!("reserve={}", maximum_token_b_reserve_balance);

    let token_a =  ConstantCurveCalculator::calculate_amount_out(
        initial_price,
        1_000_000_000,
        curve::TradeDirection::BtoA
    );

    println!(
        "token a={}",
       token_a
    );
    println!(
        "token b={}",
        ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            token_a,
           TradeDirection::AtoB
        )
    );
}
