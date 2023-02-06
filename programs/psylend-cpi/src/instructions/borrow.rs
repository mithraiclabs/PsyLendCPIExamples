use crate::{constants::*, utils::get_function_hash, Amount};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct Borrow<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The obligation with collateral to borrow with
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The reserve being borrowed from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve's vault where the borrowed tokens will be transferred from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the loan notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub loan_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that is borrowing
    pub borrower: Signer<'info>,

    /// The account to track the borrower's balance to repay
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub loan_account: UncheckedAccount<'info>,

    /// The token account that the borrowed funds will be transferred to
    /// Note: does NOT check this account is owned by the borrower. Use caution!
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub receiver_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Borrow>, bump: u8, amount: Amount) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = get_cpi_instruction(&ctx, psylend_program_id, bump, amount)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.loan_note_mint.to_account_info(),
        ctx.accounts.borrower.to_account_info(),
        ctx.accounts.loan_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    let seeds = &[
        b"loan".as_ref(),
        &ctx.accounts.reserve.key().to_bytes()[..],
        &ctx.accounts.obligation.key().to_bytes()[..],
        &ctx.accounts.borrower.key().to_bytes()[..],
    ];
    let signers_seeds = &[&seeds[..]];

    invoke_signed(&instruction, &account_infos, signers_seeds)?;
    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<Borrow>,
    program_id: Pubkey,
    bump: u8,
    amount: Amount,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new(ctx.accounts.reserve.key(), false),
            AccountMeta::new(ctx.accounts.vault.key(), false),
            AccountMeta::new(ctx.accounts.loan_note_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.borrower.key(), true),
            AccountMeta::new(ctx.accounts.loan_account.key(), false),
            AccountMeta::new(ctx.accounts.receiver_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_ix_data(bump, amount),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct CpiArgs {
    bump: u8,
    amount: Amount,
}

fn get_ix_data(bump: u8, amount: Amount) -> Vec<u8> {
    let hash = get_function_hash("global", "borrow");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = CpiArgs { bump, amount };
    args.serialize(&mut buf).unwrap();
    buf
}
