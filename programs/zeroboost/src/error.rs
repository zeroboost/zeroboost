use anchor_lang::prelude::*;

#[error_code]
pub enum MintTokenError {
    #[msg("Liquidity percentage can't be less than 0 or greater than 100")]
    InvalidLiquidityPercentage,
    #[msg("Account not own by pyth oracle program")]
    InvalidFeedAccount,
}

#[error_code]
pub enum SwapTokenError {
    #[msg("Invalid trade direction")]
    InvalidTradeDirection,
    #[msg("Mint is not tradeable on zeroboost")]
    NotTradeable,
    #[msg("Amount must be a value greater than zero")]
    InvalidAmount,
    #[msg("mint supply is empty")]
    EmptySupply,
}

#[error_code]
pub enum MigrateFundError {
    #[msg("Mint not migratable")]
    NotMigratable,
    #[msg("Mint already migrated")]
    AlreadyMigrated,
}
