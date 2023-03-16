use crate::{constants::*, utils::get_function_hash, Amount};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve associated with the c-tokens that are being withdrawn
    /// CHECK: Checked by PsyLend
    #[account()]
    pub reserve: UncheckedAccount<'info>,

    /// The obligation the collateral is being withdrawn from
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub obligation: UncheckedAccount<'info>,

    /// The user/wallet that owns the deposited collateral (depositor)
    pub owner: Signer<'info>,

    /// The account that stores the user's deposit notes, where
    /// the collateral will be returned to.
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    /// The account that contains the collateral to be withdrawn
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub collateral_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<WithdrawCollateral>,
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = withdraw_collateral_cpi_instruction(
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

pub fn withdraw_collateral_cpi_instruction(
    ctx: &Context<WithdrawCollateral>,
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
        data: withdraw_collateral_ix_data(collateral_bump, deposit_bump, amount),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawCollateralCpiArgs {
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
}

pub fn withdraw_collateral_ix_data(
    collateral_bump: u8,
    deposit_bump: u8,
    amount: Amount,
) -> Vec<u8> {
    let hash = get_function_hash("global", "withdraw_collateral");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = WithdrawCollateralCpiArgs {
        collateral_bump,
        deposit_bump,
        amount,
    };
    args.serialize(&mut buf).unwrap();
    buf
}

/// Build a CPI instruction. Accounts must be in the same order as Context
/// `WithdrawCollateral`
pub fn withdraw_collateral_cpi_ix(
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
        data: withdraw_collateral_ix_data(collateral_bump, deposit_bump, amount),
    };
    Ok(instruction)
}
