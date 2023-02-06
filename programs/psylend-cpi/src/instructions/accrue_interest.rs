use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct AccrueInterest<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve where interest is being accrued
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve's vault for storing collected fees
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub fee_note_vault: UncheckedAccount<'info>,

    /// The reserve's mint for deposit notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<AccrueInterest>) -> Result<()> {
    let program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.reserve.key(), false),
            AccountMeta::new(ctx.accounts.fee_note_vault.key(), false),
            AccountMeta::new(ctx.accounts.deposit_note_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_function_hash("global", "accrue_interest").to_vec(),
    };
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.fee_note_vault.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}