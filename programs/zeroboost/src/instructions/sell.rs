use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer_checked, Burn, Mint, Token, TokenAccount, TransferChecked},
};
use curve::{
    curve::{constant_curve::ConstantCurveCalculator, CurveCalculator, TradeDirection},
    safe_number::safe_number::NewSafeNumber,
};

use crate::{
    error::SwapTokenError,
    events::SwapEvent,
    states::{bounding_curve::BoundingCurve, position::Position},
    utils::Validate,
    CURVE_RESERVE_SEED, CURVE_SEED, POSITION_SEED,
};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(address=bounding_curve.mint)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(address=bounding_curve.pair.mint)]
    pub pair: Box<Account<'info, Mint>>,
    #[account(mut, address=position.token)]
    pub token: Box<Account<'info, Mint>>,
    #[account(
      mut,
      address=position.bounding_curve,
      seeds=[mint.key().as_ref(), CURVE_SEED.as_bytes()],
      bump
    )]
    pub bounding_curve: Box<Account<'info, BoundingCurve>>,
    #[account(
      mut,
      seeds=[bounding_curve.key().as_ref(), CURVE_RESERVE_SEED.as_bytes()],
      bump,
      owner=system_program.key()
    )]
    /// CHECK: bounding curve extra layer account for token reserve
    pub bounding_curve_reserve: UncheckedAccount<'info>,
    #[account(
      init_if_needed,
      payer=payer,
      associated_token::mint=mint,
      associated_token::authority=bounding_curve_reserve
    )]
    pub bounding_curve_reserve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
      init_if_needed,
      payer=payer,
      associated_token::mint=pair,
      associated_token::authority=bounding_curve_reserve
    )]
    pub bounding_curve_reserve_pair_ata: Box<Account<'info, TokenAccount>>,
    #[account(
      seeds=[token.key().as_ref(), POSITION_SEED.as_bytes()],
      bump
    )]
    pub position: Box<Account<'info, Position>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
      init_if_needed,
      payer=payer,
      associated_token::mint=mint,
      associated_token::authority=payer
    )]
    pub payer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      associated_token::mint=token,
      associated_token::authority=payer
    )]
    pub payer_token_ata: Box<Account<'info, TokenAccount>>,
    #[account(
      init_if_needed,
      payer=payer,
      associated_token::mint=pair,
      associated_token::authority=payer
    )]
    pub payer_pair_ata: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SellParams {
    pub amount: u64,
}

impl Validate for SellParams {
    fn validate(&self) -> Result<()> {
        if self.amount <= 0 {
            return err!(SwapTokenError::InvalidAmount);
        }
        Ok(())
    }
}

impl<'info> Sell<'info> {
    pub fn process_sell(context: Context<Self>, params: SellParams) -> Result<()> {
        params.validate()?;

        let Context {
            accounts:
                Sell {
                    token,
                    mint,
                    pair,
                    payer,
                    payer_ata,
                    payer_pair_ata,
                    payer_token_ata,
                    token_program,
                    bounding_curve,
                    bounding_curve_reserve,
                    bounding_curve_reserve_ata,
                    bounding_curve_reserve_pair_ata,
                    system_program,
                    ..
                },
            bumps,
            ..
        } = context;

        let pair_amount = ConstantCurveCalculator::calculate_amount_out(
            f64::new(bounding_curve.price),
            params.amount,
            TradeDirection::AtoB,
        );

        let token_amount = ConstantCurveCalculator::calculate_amount_out(
            f64::new(bounding_curve.price),
            pair_amount,
            TradeDirection::BtoA,
        );

        bounding_curve.sub(pair.key(), pair_amount);
        bounding_curve.add(mint.key(), token_amount);

        let bounding_curve_key = bounding_curve.key();

        let signer_seeds = &[
            bounding_curve_key.as_ref(),
            CURVE_RESERVE_SEED.as_ref(),
            &[bumps.bounding_curve_reserve],
        ];

        let signer_seeds = &[&signer_seeds[..]];

        if bounding_curve.tradeable {
            if bounding_curve.pair.is_native {
                system_program::transfer(
                    CpiContext::new_with_signer(
                        system_program.to_account_info(),
                        system_program::Transfer {
                            to: payer.to_account_info(),
                            from: bounding_curve_reserve.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    pair_amount,
                )?;
            } else {
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
                    pair_amount,
                    pair.decimals,
                )?;
            }
        } else {
            transfer_checked(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    TransferChecked {
                        mint: mint.to_account_info(),
                        to: payer_ata.to_account_info(),
                        from: bounding_curve_reserve_ata.to_account_info(),
                        authority: bounding_curve_reserve.to_account_info(),
                    },
                    signer_seeds,
                ),
                pair_amount,
                mint.decimals,
            )?;
        }

        burn(
            CpiContext::new(
                token_program.to_account_info(),
                Burn {
                    mint: token.to_account_info(),
                    from: payer_token_ata.to_account_info(),
                    authority: payer.to_account_info(),
                },
            ),
            token_amount,
        )?;
        
        let clock = Clock::get()?;

        emit!(SwapEvent {
            token_amount,
            pair_amount,
            token: token.key(),
            mint: mint.key(),
            trade_direction: 1,
            payer: payer.key(),
            timestamp: clock.unix_timestamp,
            virtual_pair_balance: bounding_curve.virtual_pair_balance,
            virtual_token_balance: bounding_curve.virtual_token_balance,
        });

        Ok(())
    }
}
