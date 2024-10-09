use anchor_lang::prelude::*;

#[event]
pub struct MintEvent {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub supply: u64,
    pub decimals: u8,
    pub bounding_curve: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SwapEvent {
    pub token: Pubkey,
    pub mint: Pubkey,
    pub token_amount: u64,
    pub pair_amount: u64,
    pub virtual_token_balance: u64,
    pub virtual_pair_balance: u64,
    pub trade_direction: u8,
    pub payer: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MigrateTriggerEvent {
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MigrateEvent {
    pub mint: Pubkey,
    pub timestamp: i64,
}
