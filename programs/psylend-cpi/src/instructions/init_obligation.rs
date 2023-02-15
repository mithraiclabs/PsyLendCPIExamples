use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct InitializeObligation<'info> {
    /// The market the obligation falls under. One obligation exists per user per market.
    /// CHECK: Checked by PsyLend
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The user/authority that is responsible for owning this obligation,
    /// typically the user's wallet.
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// The new account to track information about the borrower's loan,
    /// such as the collateral put up. A pda derived from "obligation",
    /// the market key, and the signer's key
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The SPL token program
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<InitializeObligation>, bump: u8) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = get_cpi_instruction(&ctx, psylend_program_id, bump)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.borrower.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    let seeds = &[
        b"obligation".as_ref(),
        &ctx.accounts.market.key().to_bytes()[..],
        &ctx.accounts.borrower.key().to_bytes()[..],
    ];
    let signers_seeds = &[&seeds[..]];

    invoke_signed(&instruction, &account_infos, signers_seeds)?;
    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<InitializeObligation>,
    program_id: Pubkey,
    bump: u8,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.borrower.key(), true),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
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
    let hash = get_function_hash("global", "init_obligation");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = CpiArgs { bump };
    args.serialize(&mut buf).unwrap();
    buf
}
