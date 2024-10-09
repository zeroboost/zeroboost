use anchor_lang::prelude::*;

pub const PAIR_SIZE: usize = 8 + 1 + 32;

pub const BOUNDING_CURVE_SIZE: usize = 8 + 1 + 1 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 32 + 32 + PAIR_SIZE;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum MigrationTarget {
    Raydium = 0,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct Pair {
    pub mint: Pubkey,
    pub is_native: bool,
}

impl Pair {
    pub fn new(mint: Pubkey, is_native: bool) -> Self {
        Pair { mint, is_native }
    }
}

#[account]
pub struct BoundingCurve {
    pub pair: Pair,                 // 41
    pub mint: Pubkey,               // 32
    pub price: f64,                 // 8
    pub supply: u64,                // 8
    pub migrated: bool,             // 1
    pub tradeable: bool,            // 1
    pub liquidity_percentage: u8,   // 1
    pub minimum_pair_balance: u64,  // 8
    pub maximum_pair_balance: u64,  // 8
    pub virtual_token_balance: u64, // 8
    pub virtual_pair_balance: u64,  // 8
}

impl BoundingCurve {
    pub fn add(&mut self, mint: Pubkey, amount: u64) {
        if mint.eq(&self.mint) {
            self.virtual_token_balance += amount;
        } else if mint.eq(&self.pair.mint) {
            self.virtual_pair_balance += amount;
        }
    }

    pub fn sub(&mut self, mint: Pubkey, amount: u64) {
        if mint.eq(&self.mint) {
            self.virtual_token_balance -= amount;
        } else if mint.eq(&self.pair.mint) {
            self.virtual_pair_balance -= amount;
        }
    }

    pub fn copy(&self) -> Box<BoundingCurve> {
        Box::new(BoundingCurve {
            pair: self.pair,
            mint: self.mint,
            price: self.price,
            supply: self.supply,
            migrated: self.migrated,
            tradeable: self.tradeable,
            liquidity_percentage: self.liquidity_percentage,
            minimum_pair_balance: self.maximum_pair_balance,
            maximum_pair_balance: self.maximum_pair_balance,
            virtual_token_balance: self.virtual_token_balance,
            virtual_pair_balance: self.virtual_pair_balance,
        })
    }
}
