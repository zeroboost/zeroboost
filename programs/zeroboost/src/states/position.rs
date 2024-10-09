use anchor_lang::prelude::*;

pub const POSITION_SIZE: usize = 8 + 8 + 4 + 32 + 32 + 32 + 8;

#[account]
pub struct Position {
    pub price: f64, // 8
    pub token: Pubkey, // 32
    pub bounding_curve: Pubkey, // 32
    pub creator: Pubkey, // 32
    pub timestamp: i64, // 8
}
