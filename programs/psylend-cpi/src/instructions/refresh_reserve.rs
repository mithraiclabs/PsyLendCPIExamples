use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};

use crate::constants::PSYLEND_PROGRAM_KEY;
use crate::utils::get_function_hash;

#[derive(Accounts)]
pub struct RefreshReserve<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub market: UncheckedAccount<'info>,

    /// The reserve being refreshed
    /// CHECK: Checked by PsyLend
    pub reserve: UncheckedAccount<'info>,

    /// The account containing the Pyth price information for the token.
    /// CHECK: Checked by PsyLend
    pub pyth_oracle_price: UncheckedAccount<'info>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<RefreshReserve>) -> Result<()> {
    let program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve.key(), false),
            AccountMeta::new_readonly(ctx.accounts.pyth_oracle_price.key(), false),
        ],
        data: get_function_hash("global", "refresh_reserve").to_vec(),
    };
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.pyth_oracle_price.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}
