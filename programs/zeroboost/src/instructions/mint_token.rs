use std::ops::Mul;

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked},
};

use curve::{
    curve::{constant_curve::ConstantCurveCalculator, CurveCalculator},
    safe_number::safe_number::Math,
};
use pyth_sdk_solana::state::SolanaPriceAccount;

use crate::{
    error::MintTokenError,
    events::MintEvent,
    metadata_fee_reciever, pyth,
    states::{
        bounding_curve::{BoundingCurve, MigrationTarget, BOUNDING_CURVE_SIZE},
        config::Config,
    },
    utils::{price_to_number, Validate},
    CONFIG_SEED, CURVE_RESERVE_SEED, CURVE_SEED,
};

#[derive(Accounts)]
#[instruction(params: MintTokenParams)]
pub struct MintToken<'info> {
    #[account(
        init,
        seeds = [params.name.as_ref(), params.symbol.as_ref(), creator.key().as_ref()],
        bump,
        payer = creator,
        mint::decimals = params.decimals,
        mint::authority = bounding_curve,
        mint::freeze_authority = bounding_curve,
        token::token_program = token_program
    )]
    mint: Box<Account<'info, Mint>>,
    pair: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [mint.key().as_ref(), CURVE_SEED.as_bytes()],
        bump,
        payer = creator,
        space = BOUNDING_CURVE_SIZE
    )]
    bounding_curve: Box<Account<'info, BoundingCurve>>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = bounding_curve,
        associated_token::token_program = token_program
    )]
    bounding_curve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        seeds = [bounding_curve.key().as_ref(), CURVE_RESERVE_SEED.as_bytes()],
        bump,
        space = 0,
        payer = creator,
        owner=system_program.key()
    )]
    /// CHECK: bounding curve extra layer account for token reserves
    bounding_curve_reserve: UncheckedAccount<'info>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = bounding_curve_reserve,
        associated_token::token_program = token_program
    )]
    bounding_curve_reserve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = pair,
        associated_token::authority = bounding_curve_reserve
    )]
    bounding_curve_reserve_pair_ata: Box<Account<'info, TokenAccount>>,
    #[account(seeds=[CONFIG_SEED.as_bytes()], bump)]
    config: Box<Account<'info, Config>>,
    #[account(
         mut,
        seeds= [
            b"metadata",
            &anchor_spl::metadata::ID.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
       seeds::program= &anchor_spl::metadata::ID,
     )]
    /// CHECK: metadata account
    metadata: UncheckedAccount<'info>,
    /// CHECK:
    #[account(owner=pyth::ID @ MintTokenError::InvalidFeedAccount)]
    pyth_pair_usd_feed: UncheckedAccount<'info>,
    #[account(mut, address=metadata_fee_reciever::id())]
    metadata_fee_reciever: UncheckedAccount<'info>,
    #[account(mut)]
    creator: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_metadata_program: Program<'info, Metadata>,
    rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MintTokenParams {
    name: String,
    symbol: String,
    uri: String,
    supply: u64,
    decimals: u8,
    liquidity_percentage: u8,
    migration_target: MigrationTarget,
}

impl MintTokenParams {
    pub fn validate_liquidity_percentage(&self) -> Result<()> {
        if self.liquidity_percentage > 100 {
            return err!(MintTokenError::InvalidLiquidityPercentage);
        }

        Ok(())
    }
}

impl Validate for MintTokenParams {
    fn validate(&self) -> Result<()> {
        return self.validate_liquidity_percentage();
    }
}

impl<'info> MintToken<'info> {
    pub fn process_mint_token(context: Context<MintToken>, params: &MintTokenParams) -> Result<()> {
        params.validate()?;
        let Context {
            bumps,
            accounts:
                MintToken {
                    mint,
                    pair,
                    bounding_curve_ata,
                    bounding_curve_reserve_ata,
                    pyth_pair_usd_feed,
                    config,
                    metadata,
                    creator,
                    metadata_fee_reciever,
                    rent,
                    token_program,
                    token_metadata_program,
                    system_program,
                    ..
                },
            ..
        } = context;

        let bounding_curve = &mut context.accounts.bounding_curve;

        let mint_key = &mint.key();
        let signer_seeds = &[
            mint_key.as_ref(),
            CURVE_SEED.as_bytes(),
            &[bumps.bounding_curve],
        ];
        let signer_seeds = [&signer_seeds[..]];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: mint.to_account_info(),
                    to: bounding_curve_ata.to_account_info(),
                    authority: bounding_curve.to_account_info(),
                },
                &signer_seeds,
            ),
            params.supply,
        )?;

        let feed = SolanaPriceAccount::account_info_to_feed(&pyth_pair_usd_feed).unwrap();
        let pair_usd_price = price_to_number(feed.get_price_unchecked());

        let metadata_fee: u64 = (config.metadata_creation_fee as u64).mul(10_u64.pow(5));

        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: creator.to_account_info(),
                    to: metadata_fee_reciever.to_account_info(),
                },
            ),
            metadata_fee,
        )?;

        let maximum_curve_pair_valuation: u64 = pair_usd_price
            .inverse_div(config.maximum_curve_usd_valuation.into())
            .mul((10_u128).pow(9))
            .unwrap();

        let minimum_curve_pair_valuation: u64 = pair_usd_price
            .inverse_div(config.minimum_curve_usd_valuation.into())
            .mul((10_u128).pow(9))
            .unwrap();

        let curve = ConstantCurveCalculator::new(
            params.supply,
            params.liquidity_percentage,
            maximum_curve_pair_valuation,
        );

        let initial_price = curve.calculate_initial_price();
        let bounding_curve_supply = curve.get_bounding_curve_supply().round() as u64;
        let maximum_pair_balance = curve.get_token_b_reserve_balance().round() as u64;

        bounding_curve.migrated = false;
        bounding_curve.tradeable = true;
        bounding_curve.pair = pair.key();
        bounding_curve.mint = mint.key();
        bounding_curve.initial_price = initial_price.unwrap::<f64>();
        bounding_curve.initial_supply = bounding_curve_supply;
        bounding_curve.liquidity_percentage = params.liquidity_percentage;
        bounding_curve.minimum_pair_balance = minimum_curve_pair_valuation;
        bounding_curve.maximum_pair_balance = maximum_pair_balance;
        bounding_curve.virtual_token_balance = bounding_curve_supply;
        bounding_curve.virtual_pair_balance = minimum_curve_pair_valuation;

        transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                TransferChecked {
                    mint: mint.to_account_info(),
                    from: bounding_curve_ata.to_account_info(),
                    to: bounding_curve_reserve_ata.to_account_info(),
                    authority: bounding_curve.to_account_info(),
                },
                &signer_seeds,
            ),
            bounding_curve_supply,
            params.decimals,
        )?;

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: creator.to_account_info(),
                    mint: mint.to_account_info(),
                    rent: rent.to_account_info(),
                    system_program: system_program.to_account_info(),
                    metadata: metadata.to_account_info(),
                    mint_authority: bounding_curve.to_account_info(),
                    update_authority: bounding_curve.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: params.name.clone(),
                symbol: params.symbol.clone(),
                uri: params.uri.clone(),
                uses: None,
                creators: None,
                collection: None,
                seller_fee_basis_points: 0,
            },
            false,
            true,
            None,
        )?;

        let clock = Clock::get()?;

        emit!(MintEvent {
            mint: mint.key(),
            name: params.name.clone(),
            symbol: params.symbol.clone(),
            uri: params.uri.clone(),
            supply: params.supply,
            decimals: pair.decimals,
            bounding_curve: bounding_curve.key(),
            creator: creator.key(),
            timestamp: clock.unix_timestamp,
        });
        Ok(())
    }
}
