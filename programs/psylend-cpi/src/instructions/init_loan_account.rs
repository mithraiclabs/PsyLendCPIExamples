use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        program::invoke,
    },
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct InitializeLoanAccount<'info> {
    /// The relevant market this loan is for
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The obligation the loan account is used for
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The reserve that the loan comes from
    /// CHECK: Checked by PsyLend
    #[account()]
    pub reserve: UncheckedAccount<'info>,

    /// The mint for the loan notes being used as loan
    /// CHECK: Checked by PsyLend
    pub loan_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that owns the loan
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The account that will store the loan notes.
    /// A pda derived from "loan", the reserve key, the obligation key, and the owner key
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub loan_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<InitializeLoanAccount>, bump: u8) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = get_cpi_instruction(&ctx, psylend_program_id, bump)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.loan_note_mint.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.loan_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;

    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<InitializeLoanAccount>,
    program_id: Pubkey,
    bump: u8,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve.key(), false),
            AccountMeta::new_readonly(ctx.accounts.loan_note_mint.key(), false),
            AccountMeta::new(ctx.accounts.owner.key(), true),
            AccountMeta::new(ctx.accounts.loan_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.rent.key(), false),
        ],
        data: get_ix_data(bump),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct CpiArgs {
    bump: u8,
}

fn get_ix_data(bump: u8) -> Vec<u8> {
    let hash = get_function_hash("global", "init_loan_account");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = CpiArgs { bump };
    args.serialize(&mut buf).unwrap();
    buf
}
