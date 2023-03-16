/*
    A deposit instruction is always preceded by an accrue_interest within a certain number of slots.

    An integrator that wants to deposit by CPI can either send the accrue ix seperately, or bake
    it into the same CPI, as this example demonstrates.
*/

use crate::{
    constants::*,
    instructions::{accrue_cpi_ix, deposit_tokens_cpi_ix},
    Amount,
};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct AccrueAndDepositTokens<'info> {
    /// The relevant market this deposit is for
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve being deposited into
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve's vault where the deposited tokens will be transferred to
    /// A token account holding the token
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the deposit notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that owns the deposit account
    pub depositor: Signer<'info>,

    /// The token account that will store the deposit notes
    ///
    /// Note: The only difference between this ix and `deposit` is that this ix does not perform a
    /// check on the PDA here. This allows any token account to claim the deposit notes.
    ///
    /// CHECK: Checked by PsyLend (mint only)
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    /// The token account with the tokens to be deposited
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_source: UncheckedAccount<'info>,

    /// The reserve's vault for storing collected fees
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub fee_note_vault: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<AccrueAndDepositTokens>, amount: Amount) -> Result<()> {
    // Invoke the accrue ix first
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.fee_note_vault.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];
    let ix = accrue_cpi_ix(&account_infos, psylend_program_id)?;
    invoke(&ix, &account_infos)?;

    // Invoke the deposit tokens ix after the accrue
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.deposit_source.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];
    let instruction: Instruction =
        deposit_tokens_cpi_ix(&account_infos, psylend_program_id, amount)?;

    invoke(&instruction, &account_infos)?;
    Ok(())
}
