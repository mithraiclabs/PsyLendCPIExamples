use crate::{constants::*, utils::get_function_hash, Amount};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve that the collateral comes from
    /// CHECK: Checked by PsyLend
    #[account()]
    pub reserve: UncheckedAccount<'info>,

    /// The obligation the collateral is being deposited toward
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The user/wallet that owns the deposit
    pub owner: Signer<'info>,

    /// The account that stores user deposit notes NOT used as collateral
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    /// The account that will store the deposit notes used as collateral
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub collateral_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<DepositCollateral>,
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = deposit_collateral_cpi_instruction(
        &ctx,
        psylend_program_id,
        collateral_bump,
        deposit_bump,
        amount,
    )?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.obligation.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.collateral_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;
    Ok(())
}

pub fn deposit_collateral_cpi_instruction(
    ctx: &Context<DepositCollateral>,
    program_id: Pubkey,
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new_readonly(ctx.accounts.owner.key(), true),
            AccountMeta::new(ctx.accounts.deposit_account.key(), false),
            AccountMeta::new(ctx.accounts.collateral_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: deposit_collateral_ix_data(collateral_bump, deposit_bump, amount),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositCollateralCpiArgs {
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
}

pub fn deposit_collateral_ix_data(
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
) -> Vec<u8> {
    let hash = get_function_hash("global", "deposit_collateral");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = DepositCollateralCpiArgs {
        collateral_bump,
        deposit_bump,
        amount,
    };
    args.serialize(&mut buf).unwrap();
    buf
}

/// Build a CPI instruction. Accounts must be in the same order as Context
/// `DepositCollateral`
pub fn deposit_collateral_cpi_ix(
    account_infos: &[AccountInfo; 9],
    program_id: Pubkey,
    amount: Amount,
    collateral_bump: u8,
    deposit_bump: u8,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(account_infos[0].key(), false),
            AccountMeta::new_readonly(account_infos[1].key(), false),
            AccountMeta::new_readonly(account_infos[2].key(), false),
            AccountMeta::new(account_infos[3].key(), false),
            AccountMeta::new_readonly(account_infos[4].key(), true),
            AccountMeta::new(account_infos[5].key(), false),
            AccountMeta::new(account_infos[6].key(), false),
            AccountMeta::new_readonly(account_infos[7].key(), false),
        ],
        data: deposit_collateral_ix_data(collateral_bump, deposit_bump, amount),
    };
    Ok(instruction)
}
