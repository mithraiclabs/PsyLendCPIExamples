use crate::{constants::*, utils::get_function_hash, Amount};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    /// The market the reserves fall under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The obligation with debt to be repaid
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The reserve that the debt is from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// The reserve the collateral is from
    /// CHECK: Checked by PsyLend
    pub collateral_reserve: UncheckedAccount<'info>,

    /// The reserve's vault where the payment will be transferred to
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// The mint for the debt/loan notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub loan_note_mint: UncheckedAccount<'info>,

    /// The account that holds the borrower's debt balance
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub loan_account: UncheckedAccount<'info>,

    /// The account that holds the borrower's collateral
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub collateral_account: UncheckedAccount<'info>,

    /// The token account that the payment funds will be transferred from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub payer_account: UncheckedAccount<'info>,

    /// The liquidator's own obligation, which will seize the collateral notes
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub receiver_obligation: UncheckedAccount<'info>,

    /// The account that will receive a portion of the borrower's collateral
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub receiver_account: UncheckedAccount<'info>,

    /// The account paying off the loan
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Liquidate>, amount: Amount, min_collateral: u64) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction =
        get_cpi_instruction(&ctx, psylend_program_id, amount, min_collateral)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.collateral_reserve.to_account_info(),
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.loan_note_mint.to_account_info(),
        ctx.accounts.loan_account.to_account_info(),
        ctx.accounts.collateral_account.to_account_info(),
        ctx.accounts.payer_account.to_account_info(),
        ctx.accounts.receiver_obligation.to_account_info(),
        ctx.accounts.receiver_account.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<Liquidate>,
    program_id: Pubkey,
    amount: Amount,
    min_collateral: u64,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new(ctx.accounts.reserve.key(), false),
            AccountMeta::new_readonly(ctx.accounts.collateral_reserve.key(), false),
            AccountMeta::new(ctx.accounts.vault.key(), false),
            AccountMeta::new(ctx.accounts.loan_note_mint.key(), false),
            AccountMeta::new(ctx.accounts.loan_account.key(), false),
            AccountMeta::new(ctx.accounts.collateral_account.key(), false),
            AccountMeta::new(ctx.accounts.payer_account.key(), false),
            AccountMeta::new(ctx.accounts.receiver_obligation.key(), false),
            AccountMeta::new(ctx.accounts.receiver_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.payer.key(), true),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_ix_data(amount, min_collateral),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct CpiArgs {
    amount: Amount,
    min_collateral: u64,
}

fn get_ix_data(amount: Amount, min_collateral: u64) -> Vec<u8> {
    let hash = get_function_hash("global", "liquidate");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = CpiArgs {
        amount,
        min_collateral,
    };
    args.serialize(&mut buf).unwrap();
    buf
}
