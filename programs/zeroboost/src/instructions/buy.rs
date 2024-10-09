use anchor_lang::{prelude::*, system_program};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked},
};
use curve::{
    curve::{constant_curve::ConstantCurveCalculator, CurveCalculator, TradeDirection},
    safe_number::safe_number::NewSafeNumber,
};

use crate::{
    error::SwapTokenError,
    states::{
        bounding_curve::BoundingCurve,
        position::{Position, POSITION_SIZE},
    },
    utils::Validate,
    CURVE_RESERVE_SEED, CURVE_SEED, POSITION_SEED,
};

#[derive(Accounts)]
#[instruction(param: BuyParams)]
pub struct Buy<'info> {
    #[account(
      mint::decimals = mint.decimals,
      mint::authority = bounding_curve_reserve,
      mint::freeze_authority=bounding_curve_reserve,
      token::token_program = token_program
    )]
    token: Box<Account<'info, Mint>>,
    #[account(address = bounding_curve.mint)]
    mint: Box<Account<'info, Mint>>,
    #[account(address = bounding_curve.pair.mint)]
    pair: Box<Account<'info, Mint>>,
    #[account(
      init,
      payer=payer,
      seeds=[token.key().as_ref(), POSITION_SEED.as_bytes()],
      bump,
      space=POSITION_SIZE
    )]
    position: Box<Account<'info, Position>>,
    #[account(
        mut,
        seeds = [mint.key().as_ref(), CURVE_SEED.as_bytes()],
        bump,
    )]
    bounding_curve: Box<Account<'info, BoundingCurve>>,
    #[account(
      mut,
      seeds = [bounding_curve.key().as_ref(), CURVE_RESERVE_SEED.as_bytes()],
      bump,
      owner=system_program.key()
    )]
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
        associated_token::mint = token,
        associated_token::authority = payer
    )]
    payer_token_ata: Box<Account<'info, TokenAccount>>,
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
pub struct BuyParams {
    amount: u64,
}

impl Validate for BuyParams {
    fn validate(&self) -> Result<()> {
        if self.amount <= 0 {
            return err!(SwapTokenError::InvalidAmount);
        }
        Ok(())
    }
}

impl<'info> Buy<'info> {
    pub fn process_buy(context: Context<Buy>, params: &BuyParams) -> Result<()> {
        params.validate()?;

        let Context {
            accounts:
                Buy {
                    mint,
                    pair,
                    token,
                    payer,
                    payer_token_ata,
                    payer_pair_ata,
                    bounding_curve,
                    bounding_curve_reserve,
                    bounding_curve_reserve_pair_ata,
                    position,
                    token_program,
                    system_program,
                    ..
                },
            bumps,
            ..
        } = context;

        if !bounding_curve.tradeable {
            return err!(SwapTokenError::NotTradeable);
        }

        let initial_price = f64::new(bounding_curve.price);

        let token_amount = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            params.amount,
            TradeDirection::BtoA,
        );

        let pair_amount = ConstantCurveCalculator::calculate_amount_out(
            initial_price,
            token_amount,
            TradeDirection::AtoB,
        );

        bounding_curve.sub(mint.key(), token_amount);
        bounding_curve.add(pair.key(), pair_amount);

        if bounding_curve.pair.is_native {
            system_program::transfer(
                CpiContext::new(
                    system_program.to_account_info(),
                    system_program::Transfer {
                        from: payer.to_account_info(),
                        to: bounding_curve_reserve.to_account_info(),
                    },
                ),
                pair_amount,
            )?;
        } else {
            transfer_checked(
                CpiContext::new(
                    token_program.to_account_info(),
                    TransferChecked {
                        mint: pair.to_account_info(),
                        from: payer_pair_ata.to_account_info(),
                        to: bounding_curve_reserve_pair_ata.to_account_info(),
                        authority: payer.to_account_info(),
                    },
                ),
                pair_amount,
                pair.decimals,
            )?
        }

        let bounding_curve_key = bounding_curve.key();
        let signer_seeds = &[
            bounding_curve_key.as_ref(),
            CURVE_RESERVE_SEED.as_bytes(),
            &[bumps.bounding_curve_reserve],
        ];
        let signer_seeds = [&signer_seeds[..]];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: token.to_account_info(),
                    to: payer_token_ata.to_account_info(),
                    authority: bounding_curve_reserve.to_account_info(),
                },
                &signer_seeds,
            ),
            token_amount,
        )?;

        let clock = Clock::get()?;

        position.token = token.key();
        position.creator = payer.key();
        position.price = initial_price.unwrap();
        position.timestamp = clock.unix_timestamp;
        position.bounding_curve = bounding_curve.key();

        if bounding_curve.virtual_pair_balance >= bounding_curve.maximum_pair_balance {
            bounding_curve.tradeable = false;
        }

        Ok(())
    }
}
