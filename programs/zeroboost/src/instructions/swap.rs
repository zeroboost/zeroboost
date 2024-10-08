use anchor_lang::{ prelude::*, system_program::{ self, transfer } };
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        sync_native,
        transfer_checked,
        Mint,
        SyncNative,
        Token,
        TokenAccount,
        TransferChecked,
    },
};
use curve::{
    curve::{ constant_curve::ConstantCurveCalculator, CurveCalculator, TradeDirection },
    safe_number::safe_number::NewSafeNumber,
};

use crate::{
    error::SwapTokenError,
    events::{ SwapEvent, MigrateTriggerEvent },
    states::{ bounding_curve::BoundingCurve, config::Config },
    utils::Validate,
    CONFIG_SEED,
    CURVE_RESERVE_SEED,
    CURVE_SEED,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(address = bounding_curve.mint)]
    mint: Box<Account<'info, Mint>>,
    #[account(address = bounding_curve.pair)]
    pair: Box<Account<'info, Mint>>,
    #[account(seeds = [CONFIG_SEED.as_bytes()], bump = config.bump)]
    config: Box<Account<'info, Config>>,
    #[account(
        mut,
        seeds = [mint.key().as_ref(), CURVE_SEED.as_bytes()],
        bump,
    )]
    bounding_curve: Box<Account<'info, BoundingCurve>>,
    #[account(seeds = [bounding_curve.key().as_ref(), CURVE_RESERVE_SEED.as_bytes()], bump)]
    /// CHECK: bounding curve extra layer account for token reserve
    bounding_curve_reserve: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = bounding_curve_reserve
    )]
    bounding_curve_reserve_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = pair,
        associated_token::authority = bounding_curve_reserve
    )]
    bounding_curve_reserve_pair_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    payer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = pair,
        associated_token::authority = payer
    )]
    payer_pair_ata: Box<Account<'info, TokenAccount>>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SwapParams {
    amount: u64,
    trade_direction: u8,
}

impl Validate for SwapParams {
    fn validate(&self) -> Result<()> {
        if self.amount <= 0 {
            return err!(SwapTokenError::InvalidAmount);
        }
        Ok(())
    }
}

impl<'info> Swap<'info> {
    pub fn process_swap(context: Context<Swap>, params: &SwapParams) -> Result<()> {
        params.validate()?;

        if !context.accounts.bounding_curve.tradeable {
            return err!(SwapTokenError::NotTradeable);
        }

        let trade_direction = (match params.trade_direction {
            0 => Ok(TradeDirection::BtoA),
            1 => Ok(TradeDirection::AtoB),
            _ => err!(SwapTokenError::InvalidTradeDirection),
        })?;

        let (token_amount, pair_amount) = (match trade_direction {
            TradeDirection::AtoB =>
                context.accounts.process_sell(context.bumps.bounding_curve_reserve, &params),
            TradeDirection::BtoA =>
                context.accounts.process_buy(context.bumps.bounding_curve_reserve, &params),
        })?;

        let clock = Clock::get()?;

        emit!(SwapEvent {
            token_amount,
            pair_amount,
            mint: context.accounts.mint.key(),
            payer: context.accounts.payer.key(),
            trade_direction: params.trade_direction,
            virtual_token_balance: context.accounts.bounding_curve.virtual_token_balance,
            virtual_pair_balance: context.accounts.bounding_curve.virtual_pair_balance,
            market_cap: context.accounts.bounding_curve_reserve_pair_ata.amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    #[inline(never)]
    fn process_buy(&mut self, curve_bump: u8, params: &SwapParams) -> Result<(u64, u64)> {
        let bounding_curve = &mut self.bounding_curve;
        let initial_price = f64::new(bounding_curve.initial_price);

        let amount_out = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            params.amount,
            TradeDirection::BtoA
        );

        let amount_in = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            amount_out,
            TradeDirection::AtoB
        );

        let bounding_curve_key = self.bounding_curve.key();
        let signer_seeds = &[
            bounding_curve_key.as_ref(),
            CURVE_RESERVE_SEED.as_bytes(),
            &[curve_bump],
        ];
        let signer_seeds = &[&signer_seeds[..]];

        transfer(
            CpiContext::new(self.token_program.to_account_info(), system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.bounding_curve_reserve_pair_ata.to_account_info(),
            }),
            amount_in
        )?;

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    mint: self.mint.to_account_info(),
                    from: self.bounding_curve_reserve_ata.to_account_info(),
                    to: self.payer_ata.to_account_info(),
                    authority: self.bounding_curve_reserve.to_account_info(),
                },
                signer_seeds
            ),
            amount_out,
            self.mint.decimals
        )?;

        self.bounding_curve.add(self.pair.key(), amount_in);
        self.bounding_curve.sub(self.mint.key(), amount_out);

        sync_native(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                SyncNative {
                    account: self.bounding_curve_reserve_pair_ata.to_account_info(),
                },
                signer_seeds
            )
        )?;

        // One off mutation, If trade is maked as non tradeable all swap is stop until token migrated to a dex
        // when migrated token holders can continue trade with dex
        if self.bounding_curve.maximum_pair_balance >= self.bounding_curve_reserve_pair_ata.amount {
            self.bounding_curve.tradeable = false;
            let clock = Clock::get()?;
            emit!(MigrateTriggerEvent { mint: self.mint.key(), timestamp: clock.unix_timestamp });
        }

        Ok((amount_out, amount_in))
    }

    #[inline(never)]
    fn process_sell(&mut self, curve_bump: u8, params: &SwapParams) -> Result<(u64, u64)> {
        let initial_price = f64::new(self.bounding_curve.initial_price);

        let amount_out = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            params.amount,
            TradeDirection::AtoB
        );

        let amount_in = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            amount_out,
            TradeDirection::BtoA
        );

        let bounding_curve_key = self.bounding_curve.key();
        let signer_seeds = &[
            bounding_curve_key.as_ref(),
            CURVE_RESERVE_SEED.as_bytes(),
            &[curve_bump],
        ];
        let signer_seeds = &[&signer_seeds[..]];

        transfer_checked(
            CpiContext::new(self.token_program.to_account_info(), TransferChecked {
                mint: self.mint.to_account_info(),
                from: self.payer_ata.to_account_info(),
                to: self.bounding_curve_reserve_ata.to_account_info(),
                authority: self.payer.to_account_info(),
            }),
            amount_in,
            self.mint.decimals
        )?;

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    mint: self.pair.to_account_info(),
                    to: self.payer_pair_ata.to_account_info(),
                    from: self.bounding_curve_reserve_pair_ata.to_account_info(),
                    authority: self.bounding_curve_reserve.to_account_info(),
                },
                signer_seeds
            ),
            amount_out,
            self.pair.decimals
        )?;

        self.bounding_curve.sub(self.pair.key(), amount_out);
        self.bounding_curve.add(self.mint.key(), amount_in);

        sync_native(
            CpiContext::new(self.token_program.to_account_info(), SyncNative {
                account: self.payer_pair_ata.to_account_info(),
            })
        )?;

        Ok((amount_in, amount_out))
    }
}
