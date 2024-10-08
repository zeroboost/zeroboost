use anchor_lang::prelude::*;

pub mod config;
pub mod bounding_curve;


#[account]
pub struct  ZeroAccount {
  owner: Pubkey
}