use anchor_lang::prelude::*;

use crate::{ admin, states::config::{ Config, CONFIG_SIZE }, CONFIG_SEED };

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init_if_needed,
        seeds = [CONFIG_SEED.as_bytes()],
        bump,
        space = CONFIG_SIZE,
        payer = admin
    )]
    config: Account<'info, Config>,
    #[account(mut, address=admin::ID)]
    admin: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeConfigParams {
  metadata_creation_fee: u8,
    migration_percentage_fee: u8,
    minimum_curve_usd_valuation: u16,
    maximum_curve_usd_valuation: u16,
    estimated_raydium_cp_pool_fee: u64,
}

impl<'info> InitializeConfig<'info> {
    pub fn process_initialize(
        context: Context<InitializeConfig>,
        params: InitializeConfigParams
    ) -> Result<()> {
        let config = &mut context.accounts.config;

        config.bump = context.bumps.config;
        config.metadata_creation_fee = params.metadata_creation_fee;
        config.migration_percentage_fee = params.migration_percentage_fee;
        config.minimum_curve_usd_valuation = params.minimum_curve_usd_valuation;
        config.maximum_curve_usd_valuation = params.maximum_curve_usd_valuation;
        config.estimated_raydium_cp_pool_creation_fee = params.estimated_raydium_cp_pool_fee;

        Ok(())
    }
}
