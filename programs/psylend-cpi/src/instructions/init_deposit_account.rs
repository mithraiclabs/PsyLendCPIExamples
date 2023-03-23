use crate::{constants::*, utils::get_function_hash};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token::Token;
use std::str::FromStr;

#[derive(Accounts)]
pub struct InitializeDepositAccount<'info> {
    /// The market the reserve falls under
    /// CHECK: Checked by PsyLend
    #[account()]
    pub market: UncheckedAccount<'info>,

    /// The market's authority account: a pda derived from the market
    /// CHECK: Checked by PsyLend
    pub market_authority: UncheckedAccount<'info>,

    /// The reserve being deposited into
    /// CHECK: Checked by PsyLend
    #[account()]
    pub reserve: UncheckedAccount<'info>,

    /// The mint for the deposit notes
    /// CHECK: Checked by PsyLend
    pub deposit_note_mint: UncheckedAccount<'info>,

    /// The user/wallet that will own the deposit account
    /// 
    /// Note: the market authority is the owner/authority in the technical sense.
    /// 
    /// CHECK: opening a deposit acc for a depositor does not require their permission
    pub depositor: UncheckedAccount<'info>,

    /// The user/authority that pays the fees to open the deposit acc
    ///
    /// Typically the same as the depositor, unless opening an account
    /// on behalf of another user
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The account that will store the deposit notes
    /// A pda derived from "deposits", the reserve key, and the depositor key.
    /// CHECK: Checked by PsyLend
    #[account(mut)]
    pub deposit_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by constraint
    #[account(address = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap())]
    pub psylend_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<InitializeDepositAccount>, bump: u8) -> Result<()> {
    let psylend_program_id: Pubkey = Pubkey::from_str(PSYLEND_PROGRAM_KEY).unwrap();
    let instruction: Instruction = init_deposit_cpi_instruction(&ctx, psylend_program_id, bump)?;
    let account_infos = [
        ctx.accounts.market.to_account_info(),
        ctx.accounts.market_authority.to_account_info(),
        ctx.accounts.reserve.to_account_info(),
        ctx.accounts.deposit_note_mint.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.deposit_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.psylend_program.to_account_info(),
    ];

    invoke(&instruction, &account_infos)?;

    Ok(())
}

pub fn init_deposit_cpi_instruction(
    ctx: &Context<InitializeDepositAccount>,
    program_id: Pubkey,
    bump: u8,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.market_authority.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve.key(), false),
            AccountMeta::new_readonly(ctx.accounts.deposit_note_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.depositor.key(), false),
            AccountMeta::new(ctx.accounts.payer.key(), true),
            AccountMeta::new(ctx.accounts.deposit_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.rent.key(), false),
        ],
        data: init_deposit_ix_data(bump),
    };
    Ok(instruction)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitDepositCpiArgs {
    bump: u8,
}

pub fn init_deposit_ix_data(bump: u8) -> Vec<u8> {
    let hash = get_function_hash("global", "init_deposit_account");
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    let args = InitDepositCpiArgs { bump };
    args.serialize(&mut buf).unwrap();
    buf
}

/// Build a CPI instruction. Accounts must be in the same order as Context
/// `InitializeDepositAccount`
pub fn init_deposit_cpi_ix(
    account_infos: &[AccountInfo; 11],
    program_id: Pubkey,
    bump: u8,
) -> Result<Instruction> {
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(account_infos[0].key(), false),
            AccountMeta::new_readonly(account_infos[1].key(), false),
            AccountMeta::new_readonly(account_infos[2].key(), false),
            AccountMeta::new_readonly(account_infos[3].key(), false),
            AccountMeta::new_readonly(account_infos[4].key(), false),
            AccountMeta::new(account_infos[5].key(), true),
            AccountMeta::new(account_infos[6].key(), false),
            AccountMeta::new_readonly(account_infos[7].key(), false),
            AccountMeta::new_readonly(account_infos[8].key(), false),
            AccountMeta::new_readonly(account_infos[9].key(), false),
        ],
        data: init_deposit_ix_data(bump),
    };
    Ok(instruction)
}
