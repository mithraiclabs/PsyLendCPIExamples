use anchor_lang::prelude::*;

declare_id!("BVr85VrQhRJAixhUt68bmodrvzv5nQXdUMbuihRWqNGb");

pub mod constants;
pub mod instructions;
pub mod utils;
use instructions::*;

#[program]
pub mod psylend_cpi {
    use super::*;

    /// A bare-minium CPI call to a trivial program (devnet only)
    pub fn dummy_cpi(ctx: Context<DummyMsgCpi>) -> Result<()> {
        instructions::dummy_cpi::handler(ctx)
    }

    pub fn accrue_interest_cpi(ctx: Context<AccrueInterest>) -> Result<()> {
        instructions::accrue_interest::handler(ctx)
    }

    pub fn init_obligation_cpi(ctx: Context<InitializeObligation>, bump: u8) -> Result<()> {
        instructions::init_obligation::handler(ctx, bump)
    }

    pub fn refresh_reserve_cpi(ctx: Context<RefreshReserve>) -> Result<()> {
        instructions::refresh_reserve::handler(ctx)
    }

    pub fn refresh_psyfi_reserve_cpi(ctx: Context<RefreshPsyFiReserve>) -> Result<()> {
        instructions::refresh_psyfi_reserve::handler(ctx)
    }

    pub fn close_obligation_cpi(ctx: Context<CloseObligation>) -> Result<()> {
        instructions::close_obligation::handler(ctx)
    }

    pub fn close_deposit_cpi(ctx: Context<CloseDepositAccount>) -> Result<()> {
        instructions::close_deposit_account::handler(ctx)
    }

    pub fn init_deposit_cpi(ctx: Context<InitializeDepositAccount>, bump: u8) -> Result<()> {
        instructions::init_deposit_account::handler(ctx, bump)
    }

    pub fn deposit_cpi(ctx: Context<Deposit>, bump: u8, amount: Amount) -> Result<()> {
        instructions::deposit::handler(ctx, bump, amount)
    }

    pub fn withdraw_cpi(ctx: Context<Withdraw>, bump: u8, amount: Amount) -> Result<()> {
        instructions::withdraw::handler(ctx, bump, amount)
    }

    pub fn init_collateral_account_cpi(
        ctx: Context<InitializeCollateralAccount>,
        bump: u8,
    ) -> Result<()> {
        instructions::init_collateral_account::handler(ctx, bump)
    }

    pub fn close_collateral_account_cpi(ctx: Context<CloseCollateralAccount>) -> Result<()> {
        instructions::close_collateral_account::handler(ctx)
    }

    pub fn borrow_cpi(ctx: Context<Borrow>, bump: u8, amount: Amount) -> Result<()> {
        instructions::borrow::handler(ctx, bump, amount)
    }

    pub fn init_loan_account_cpi(ctx: Context<InitializeLoanAccount>, bump: u8) -> Result<()> {
        instructions::init_loan_account::handler(ctx, bump)
    }

    pub fn close_loan_account_cpi(ctx: Context<CloseLoanAccount>) -> Result<()> {
        instructions::close_loan_account::handler(ctx)
    }

    pub fn deposit_collateral_cpi(
        ctx: Context<DepositCollateral>,
        collateral_bump: u8,
        deposit_bump: u8,
        amount: Amount,
    ) -> Result<()> {
        instructions::deposit_collateral::handler(ctx, collateral_bump, deposit_bump, amount)
    }

    pub fn withdraw_collateral_cpi(
        ctx: Context<WithdrawCollateral>,
        collateral_bump: u8,
        deposit_bump: u8,
        amount: Amount,
    ) -> Result<()> {
        instructions::withdraw_collateral::handler(ctx, collateral_bump, deposit_bump, amount)
    }

    pub fn repay_cpi(ctx: Context<Repay>, amount: Amount) -> Result<()> {
        instructions::repay::handler(ctx, amount)
    }

    pub fn deposit_tokens_cpi(ctx: Context<DepositTokens>, bump: u8, amount: Amount) -> Result<()> {
        instructions::deposit_tokens::handler(ctx, bump, amount)
    }

    pub fn withdraw_tokens_cpi(
        ctx: Context<WithdrawTokens>,
        bump: u8,
        amount: Amount,
    ) -> Result<()> {
        instructions::withdraw_tokens::handler(ctx, bump, amount)
    }

    // This ix is quite large, the program may not have space for it, or you may need to Box Accounts.
    // pub fn liquidate_cpi(
    //     ctx: Context<Liquidate>,
    //     amount: Amount,
    //     min_collateral: u64,
    // ) -> Result<()> {
    //     instructions::liquidate::handler(ctx, amount, min_collateral)
    // }
}

pub const TOKENS: u8 = 0;
pub const DEPOSIT_NOTES: u8 = 1;
pub const LOAN_NOTES: u8 = 2;

/// Represent an amount of some value (like tokens, or notes).
/// For units, possible values are TOKENS (0), DEPOSIT_NOTES (1), and LOAN_NOTES (2)
#[derive(AnchorDeserialize, AnchorSerialize, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Amount {
    pub units: u8,
    pub value: u64,
}
