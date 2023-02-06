use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct CloseCollateralAccount<'info> {
    /// The market this collateral account is under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The obligation the collateral account is under
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The user/wallet that owns the collateral
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The account that stores the deposit notes used as collateral.
    /// Will be closed. Must be empty.
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub collateral_account: UncheckedAccount<'info>,

    /// The account that stores deposit notes NOT used as collateral
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CloseCollateralAccount>) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = get_cpi_instruction(&ctx, psylend_program_id)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.collateral_account.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}

fn get_cpi_instruction(
    ctx: &Context<CloseCollateralAccount>,
    program_id: Pubkey,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new(ctx.accounts.owner.key(), true),
            AccountMeta::new(ctx.accounts.collateral_account.key(), false),
            AccountMeta::new(ctx.accounts.deposit_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: get_function_hash("global", "close_collateral_account").to_vec(),
    };
    Ok(instruction)
}
