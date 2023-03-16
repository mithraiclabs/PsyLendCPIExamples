use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use std::str::FromStr;

#[derive(Accounts)]
pub struct CloseObligation<'info> {
    /// The market the obligation falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The user/wallet that ownsthis obligation.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// User obligation to close. Must have no collateral or loan positions active.
    /// Marks the account as being closed at the end of this instruction’s execution,
    /// sending the rent exemption lamports to the owner. Close is implicit.
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<CloseObligation>) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = close_obligation_cpi_instruction(&ctx, psylend_program_id)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}

pub fn close_obligation_cpi_instruction(
    ctx: &Context<CloseObligation>,
    program_id: Pubkey,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new(ctx.accounts.owner.key(), true),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
        ],
        data: get_function_hash("global", "close_obligation").to_vec(),
    };
    Ok(instruction)
}

/// Build a CPI instruction. Accounts must be in the same order as Context
/// `CloseObligation`
pub fn close_obligation_cpi_ix(
    account_infos: &[AccountInfo; 5],
    program_id: Pubkey,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(  account_infos[0].key(), false),
            AccountMeta::new_readonly(  account_infos[1].key(), false),
            AccountMeta::new(           account_infos[2].key(), true),
            AccountMeta::new(           account_infos[3].key(), false),
        ],
        data: get_function_hash("global", "close_obligation").to_vec(),
    };
    Ok(instruction)
}