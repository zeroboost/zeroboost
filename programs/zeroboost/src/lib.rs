use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::*;

#[cfg(not(feature="no-entrypoint"))]
solana_security_txt::security_txt!{
  name: "zeroboost",
  project_url: "https://zeroboost.fun",
  contacts: "link:https://zeroboost.fun/#contact",
  policy: "https://zeroboost.fun/#policy",
  source_code: "http://github.com/zeroboost/zeroboost",
  preferred_language: "en",
  auditors: "None"
}

#[cfg(feature = "devnet")]
declare_id!("HW7sPVEXwDyZ7WjmS52dLi6WYiWALnVbSvXYjZ9jErZq");
#[cfg(not(feature = "devnet"))]
declare_id!("Zero9JeEwbjEGE3u9d9xeAUXHbAvYTKMJ8zufMk3BeY");

pub mod admin {
    use anchor_lang::declare_id;

    #[cfg(feature = "devnet")]
    declare_id!("9meGAekj5fSks2oYbv5RmVoxUam5d9T1RaxPhofnHmV2");
    #[cfg(not(feature = "devnet"))]
    declare_id!("Hash4eNpLr5gw2VcRStzEn514fTYTkfmjFFb5bAPwB4z");
}

pub mod metadata_fee_reciever {
    use anchor_lang::declare_id;

    #[cfg(feature = "devnet")]
    declare_id!("2nAn6RP1zbSNDgkmh3atTJZn84oKkLnDDDdbruBTu4Lz");
    #[cfg(not(feature = "devnet"))]
    declare_id!("FundhjabaKMsW3VrweuvMefcSUPLgdtUX7Dv5atUxSBP");
}


pub mod migration_fee_receiver {
    use anchor_lang::declare_id;

    #[cfg(feature = "devnet")]
    declare_id!("9meGAekj5fSks2oYbv5RmVoxUam5d9T1RaxPhofnHmV2");
    #[cfg(not(feature = "devnet"))]
    declare_id!("FundQMz92akoVMxTn36yNxMiNJnecMPqKa3pTPDMA7MC");
}

pub mod pyth {
    use anchor_lang::declare_id;
    
    #[cfg(feature = "devnet")]
    declare_id!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
    #[cfg(not(feature = "devnet"))]
    declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
}

pub const CONFIG_SEED: &str = "zeroboost";
pub const CURVE_SEED: &str = "curve";
pub const CURVE_RESERVE_SEED: &str = "curve_reserve";
pub const POSITION_SEED: &str = "position";

#[program]
pub mod zeroboost {
    use super::*;

    pub fn initialize_config(
        context: Context<InitializeConfig>,
        params: InitializeConfigParams,
    ) -> Result<()> {
        InitializeConfig::process_initialize(context, params)
    }

    pub fn mint_token(context: Context<MintToken>, params: MintTokenParams) -> Result<()> {
        MintToken::process_mint_token(context, &params)
    }

    pub fn buy(context: Context<Buy>, params: BuyParams) -> Result<()> {
        Buy::process_buy(context, &params)
    }
    
    pub fn sell(context: Context<Sell>, params: SellParams) -> Result<()> {
      Sell::process_sell(context, params)
    }

    pub fn migrate_fund(context: Context<MigrateFund>, params: MigrateFundParams) -> Result<()> {
        MigrateFund::process_migrate_fund(context, &params)
    }
}
