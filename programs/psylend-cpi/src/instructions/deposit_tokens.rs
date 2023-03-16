use crate::{constants::*, utils::get_function_hash, Amount};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    /// The relevant market this deposit is for
    /// CHECK: Checked by PsyLend
    #[account()]
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

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<DepositTokens>, amount: Amount) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction =
        deposit_tokens_cpi_instruction(&ctx, psylend_program_id, amount)?;
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

    invoke(&instruction, &account_infos)?;
    Ok(())
}

pub fn deposit_tokens_cpi_instruction(
    ctx: &Context<DepositTokens>,
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
            AccountMeta::new(ctx.accounts.deposit_source.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: deposit_tokens_ix_data(amount),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositTokensCpiArgs {
    amount: Amount,
}

pub fn deposit_tokens_ix_data(amount: Amount) -> Vec<u8> {
    let hash = get_function_hash("global", "deposit_tokens");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = DepositTokensCpiArgs { amount };
    args.serialize(&mut buf).unwrap();
    buf
}

/// Build a CPI instruction. Accounts must be in the same order as Context
/// `DepositTokens`
pub fn deposit_tokens_cpi_ix(
    account_infos: &[AccountInfo; 10],
    program_id: Pubkey,
    amount: Amount,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(account_infos[0].key(), false),
            AccountMeta::new_readonly(account_infos[1].key(), false),
            AccountMeta::new(account_infos[2].key(), false),
            AccountMeta::new(account_infos[3].key(), false),
            AccountMeta::new(account_infos[4].key(), false),
            AccountMeta::new_readonly(account_infos[5].key(), true),
            AccountMeta::new(account_infos[6].key(), false),
            AccountMeta::new(account_infos[7].key(), false),
            AccountMeta::new_readonly(account_infos[8].key(), false),
        ],
        data: deposit_tokens_ix_data(amount),
    };
    Ok(instruction)
}
