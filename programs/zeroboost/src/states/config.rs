use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub bump: u8,
    pub metadata_creation_fee: u8,
    pub migration_percentage_fee: u8,
    pub minimum_curve_usd_valuation: u16,
    pub maximum_curve_usd_valuation: u16,
    pub estimated_raydium_cp_pool_creation_fee: u64,
}

pub const CONFIG_SIZE: usize = 8 + 1 + 1 + 1 + 2 + 2 + 8;
