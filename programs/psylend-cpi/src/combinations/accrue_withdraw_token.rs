/*
    A withdraw instruction is always preceded by an accrue_interest within a certain number of slots.

    An integrator that wants to withdraw by CPI can either send the accrue ix seperately, or bake
    it into the same CPI, as this example demonstrates.
*/

use crate::{
    constants::*, utils::get_function_hash, Amount,
};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct AccrueAndWithdrawTokens<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve being withdrawn from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve's vault where the withdrawn tokens will be transferred from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the deposit notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that owns the deposit.
    pub depositor: Signer<'info>,

    /// The account that stores the deposit notes
    ///
    /// Note: The only difference between this ix and `withdraw` is that this ix does not perform a
    /// check on the PDA here. This allows any token account to claim the deposit notes.
    ///
    /// CHECK: Checked by PsyLend (mint and depositor only)
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    /// The token account where to transfer withdrawn tokens to
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub withdraw_account: UncheckedAccount<'info>,

    /// The reserve's vault for storing collected fees
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub fee_note_vault: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<AccrueAndWithdrawTokens>, amount: Amount) -> Result<()> {
    // Invoke the accrue ix first
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction = Instruction {
        program_id: psylend_program_id,
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

    // Invoke the withdraw tokens ix after the accrue
    let instruction: Instruction =
        get_withdraw_token_cpi_instruction(&ctx, psylend_program_id, amount)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.withdraw_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];
    invoke(&instruction, &account_infos)?;
    Ok(())
}

fn get_withdraw_token_cpi_instruction(
    ctx: &Context<AccrueAndWithdrawTokens>,
    program_id: Pubkey,
    amount: Amount,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.reserve.key(), false),
            AccountMeta::new(ctx.accounts.vault.key(), false),
            AccountMeta::new(ctx.accounts.deposit_note_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.depositor.key(), true),
            AccountMeta::new(ctx.accounts.deposit_account.key(), false),
            AccountMeta::new(ctx.accounts.withdraw_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_withdraw_token_ix_data(amount),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct CpiArgs {
    amount: Amount,
}

fn get_withdraw_token_ix_data(amount: Amount) -> Vec<u8> {
    let hash = get_function_hash("global", "withdraw_tokens");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = CpiArgs { amount };
    args.serialize(&mut buf).unwrap();
    buf
}
