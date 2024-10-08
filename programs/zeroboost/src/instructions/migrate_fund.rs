use std::ops::{Div, Mul};

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{transfer_checked, TransferChecked};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn, Mint, Token, TokenAccount},
};
use raydium_cp_swap::{
    cpi::{accounts::Initialize, initialize},
    create_pool_fee_reveiver,
    program::RaydiumCpSwap,
    states::{OBSERVATION_SEED, POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
    AUTH_SEED,
};
use spl_token::solana_program::program_pack::Pack;

use crate::{
    events::MigrateEvent,
    error::MigrateFundError,
    states::{bounding_curve::BoundingCurve, config::Config},
    migration_fee_receiver,
    CONFIG_SEED, CURVE_RESERVE_SEED, CURVE_SEED,
};

#[derive(Accounts)]
pub struct MigrateFund<'info> {
    #[account(seeds = [CONFIG_SEED.as_bytes()], bump)]
    config: Box<Account<'info, Config>>,
    #[account(address = bounding_curve.mint)]
    mint: Box<Account<'info, Mint>>,
    #[account(address = bounding_curve.pair)]
    pair: Box<Account<'info, Mint>>,
    #[account(mut, seeds = [mint.key().as_ref(), CURVE_SEED.as_bytes()], bump)]
    bounding_curve: Box<Account<'info, BoundingCurve>>,
    #[account(mut, associated_token::mint=mint, associated_token::authority=bounding_curve)]
    bounding_curve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [bounding_curve.key().as_ref(), CURVE_RESERVE_SEED.as_bytes()],
        bump
    )]
    /// CHECK:
    bounding_curve_reserve: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bounding_curve_reserve
    )]
    bounding_curve_reserve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = pair,
        associated_token::authority = bounding_curve_reserve
    )]
    bounding_curve_reserve_pair_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds=[
            bounding_curve_reserve.key().as_ref(),
            token_program.key().as_ref(),
            amm_lp_mint.key().as_ref()
        ],
        bump,
        seeds::program=associated_token_program.key()
    )]
    /// CHECK: Bounding curve lp token account
    bounding_curve_reserve_lp_ata: UncheckedAccount<'info>,
    /// CHECK: Config the pool belongs to.
    amm_config: UncheckedAccount<'info>,
    #[account(address=Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &amm_program.key()).0)]
    /// CHECK: Pool vault and lp mint authority
    amm_authority: UncheckedAccount<'info>,
    #[account(mut, address=create_pool_fee_reveiver::ID)]
    /// CHECK:Pool fee reeciever
    amm_fee_receiver: UncheckedAccount<'info>,
    #[account(
      mut,
      address=Pubkey::find_program_address(
          &[
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            pair.key().as_ref(),
            mint.key().as_ref(),
        ],
        &amm_program.key()
      ).0
    )]
    /// CHECK: Initialize an account to store the pool state
    amm_pool_state: UncheckedAccount<'info>,
    #[account(mut,
        address=Pubkey::find_program_address(
          &[
            POOL_LP_MINT_SEED.as_bytes(),
            amm_pool_state.key().as_ref()
          ],
          &amm_program.key()
        ).0
    )]
    /// CHECK: Pool lp mint
    amm_lp_mint: UncheckedAccount<'info>,
    #[account(
      mut,
        address=Pubkey::find_program_address(
          &[
            POOL_VAULT_SEED.as_bytes(),
            amm_pool_state.key.as_ref(),
            mint.key().as_ref()
          ],
          &amm_program.key()
        ).0
    )]
    /// CHECK: Mint vault for the pool
    amm_mint_vault: UncheckedAccount<'info>,
    #[account(
      mut,
        address=Pubkey::find_program_address(
          &[
            POOL_VAULT_SEED.as_bytes(),
            amm_pool_state.key.as_ref(),
            pair.key().as_ref()
          ],
          &amm_program.key()
        ).0
    )]
    /// CHECK: Pair vault for the pool
    amm_pair_vault: UncheckedAccount<'info>,
    #[account(
        mut,
        address=Pubkey::find_program_address(
          &[
            OBSERVATION_SEED.as_bytes(),
            amm_pool_state.key.as_ref()
          ],
          &amm_program.key()
        ).0
    )]
    /// CHECK: Account to store oracle observations
    amm_observable_state: UncheckedAccount<'info>,
    #[account(mut, address=migration_fee_receiver::id())]
    payer: Signer<'info>,
    #[account(
        init_if_needed,
        associated_token::mint = pair,
        associated_token::authority = payer,
        payer = payer
    )]
    payer_pair_ata: Box<Account<'info, TokenAccount>>,
    amm_program: Program<'info, RaydiumCpSwap>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MigrateFundParams {
    open_time: Option<u64>,
}

impl<'info> MigrateFund<'info> {
    pub fn process_migrate_fund(
        context: Context<MigrateFund>,
        params: &MigrateFundParams,
    ) -> Result<()> {
        let Context {
            bumps,
            accounts:
                MigrateFund {
                    config,
                    mint,
                    pair,
                    bounding_curve_ata,
                    bounding_curve_reserve,
                    bounding_curve_reserve_ata,
                    bounding_curve_reserve_pair_ata,
                    bounding_curve_reserve_lp_ata,
                    amm_fee_receiver,
                    amm_pool_state,
                    amm_lp_mint,
                    amm_authority,
                    amm_mint_vault,
                    amm_pair_vault,
                    amm_config,
                    amm_observable_state,
                    payer,
                    payer_pair_ata,
                    rent,
                    amm_program,
                    token_program,
                    associated_token_program,
                    system_program,
                    ..
                },
            ..
        } = context;
        let bounding_curve = &mut context.accounts.bounding_curve;
        

        if bounding_curve.tradeable {
            return err!(MigrateFundError::NotMigratable);
        }

        if bounding_curve.migrated {
            return err!(MigrateFundError::AlreadyMigrated);
        }

        let bounding_curve_key = bounding_curve.key();
        let signer_seeds = &[
            bounding_curve_key.as_ref(),
            CURVE_RESERVE_SEED.as_bytes(),
            &[bumps.bounding_curve_reserve],
        ];
        let signer_seeds = &[&signer_seeds[..]];

        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: bounding_curve_reserve.to_account_info(),
                },
            ),
            config.estimated_raydium_cp_pool_creation_fee,
        )?;

        let init_amount = bounding_curve_ata.amount;
        let pair_init_amount = bounding_curve_reserve_pair_ata.amount;

        let admin_fee = pair_init_amount
            .mul(config.migration_percentage_fee as u64)
            .div(100);
        let pair_init_amount = pair_init_amount - admin_fee;

        transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                TransferChecked {
                    mint: pair.to_account_info(),
                    to: payer_pair_ata.to_account_info(),
                    from: bounding_curve_reserve_pair_ata.to_account_info(),
                    authority: bounding_curve_reserve.to_account_info(),
                },
                signer_seeds,
            ),
            admin_fee,
            pair.decimals,
        )?;
        

        let mint_key = mint.key();

        transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                TransferChecked {
                    mint: mint.to_account_info(),
                    to: bounding_curve_reserve_ata.to_account_info(),
                    from: bounding_curve_ata.to_account_info(),
                    authority: bounding_curve.to_account_info(),
                },
                &[&[
                    &mint_key.as_ref(),
                    CURVE_SEED.as_bytes(),
                    &[bumps.bounding_curve],
                ]],
            ),
            init_amount,
            pair.decimals,
        )?;

        initialize(
            CpiContext::new_with_signer(
                amm_program.to_account_info(),
                Initialize {
                    token_0_mint: pair.to_account_info(),
                    token_1_mint: mint.to_account_info(),
                    creator: bounding_curve_reserve.to_account_info(),
                    amm_config: amm_config.to_account_info(),
                    authority: amm_authority.to_account_info(),
                    pool_state: amm_pool_state.to_account_info(),
                    lp_mint: amm_lp_mint.to_account_info(),
                    creator_token_0: bounding_curve_reserve_pair_ata.to_account_info(),
                    creator_token_1: bounding_curve_reserve_ata.to_account_info(),
                    creator_lp_token: bounding_curve_reserve_lp_ata.to_account_info(),
                    token_0_vault: amm_pair_vault.to_account_info(),
                    token_1_vault: amm_mint_vault.to_account_info(),
                    token_program: token_program.to_account_info(),
                    token_0_program: token_program.to_account_info(),
                    token_1_program: token_program.to_account_info(),
                    associated_token_program: associated_token_program.to_account_info(),
                    system_program: system_program.to_account_info(),
                    rent: rent.to_account_info(),
                    observation_state: amm_observable_state.to_account_info(),
                    create_pool_fee: amm_fee_receiver.to_account_info(),
                },
                signer_seeds,
            ),
            pair_init_amount,
            init_amount,
            match params.open_time {
              Some(open_time) => open_time,
              None => 0
            },
        )?;

        let bounding_curve_reserve_lp_ata = bounding_curve_reserve_lp_ata.to_account_info();
        let bounding_curve_reserve_lp = spl_token::state::Account::unpack(
            &bounding_curve_reserve_lp_ata.data.try_borrow().unwrap(),
        )?;

        burn(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Burn {
                    mint: amm_lp_mint.to_account_info(),
                    from: bounding_curve_reserve_lp_ata,
                    authority: bounding_curve_reserve.to_account_info(),
                },
                signer_seeds,
            ),
            bounding_curve_reserve_lp.amount,
        )?;
        
        
        bounding_curve.migrated = true;

        let clock = Clock::get()?;

        emit!(MigrateEvent {
            mint: mint.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
