use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct CloseDepositAccount<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve deposited into
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve's vault, where any tokens to withdraw will be transferred from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the deposit notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that owns the deposit account
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// The account that stores the deposit notes, to be closed
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    /// The account to receive any remaining tokens still deposited
    /// Note: NO VALIDATION is conducted to assure that this is owned the depositor.
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub receiver_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CloseDepositAccount>) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = get_cpi_instruction(&ctx, psylend_program_id)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.receiver_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<CloseDepositAccount>,
    program_id: Pubkey,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.reserve.key(), false),
            AccountMeta::new(ctx.accounts.vault.key(), false),
            AccountMeta::new(ctx.accounts.deposit_note_mint.key(), false),
            AccountMeta::new(ctx.accounts.depositor.key(), true),
            AccountMeta::new(ctx.accounts.deposit_account.key(), false),
            AccountMeta::new(ctx.accounts.receiver_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_function_hash("global", "close_deposit_account").to_vec(),
    };
    Ok(instruction)
}
