use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};

use crate::utils::get_function_hash;

pub const DUMMY_PROGRAM_KEY: &str = "Ev6JrN5HqrKwXhoB9jucLdn51yzzDvWmBHkubXWavRio";

#[derive(Accounts)]
pub struct DummyMsgCpi<'info> {
    /// CHECK: Checked by dummy program
    pub dummy_acc: UncheckedAccount<'info>,
    /// CHECK: Validated by constraint
    #[account(
        address = Pubkey::from_str(DUMMY_PROGRAM_KEY).unwrap()
    )]
    pub dummy_program: UncheckedAccount<'info>, // < add
}

pub fn handler(ctx: Context<DummyMsgCpi>) -> Result<()> {
    invoke(
        &Instruction {
            program_id: Pubkey::from_str(DUMMY_PROGRAM_KEY).unwrap(),
            accounts: vec![AccountMeta::new_readonly(
                ctx.accounts.dummy_acc.key(),
                false,
            )],
            data: get_function_hash("global", "msg").to_vec(),
        },
        &[
            ctx.accounts.dummy_acc.to_account_info(),
            ctx.accounts.dummy_program.to_account_info(),
        ],
    )?;
    Ok(())
}

// Reference code for the CPI-Dummy program:

// Lib.rs
/*
    use anchor_lang::prelude::*;
    declare_id!("Ev6JrN5HqrKwXhoB9jucLdn51yzzDvWmBHkubXWavRio");
    pub mod instructions;
    pub use instructions::*;
    pub mod cpi_dummy {
        use super::*;

        pub fn msg(ctx: Context<Msg>) -> Result<()> {
            instructions::msg::handler(ctx)
        }
    }
*/

// Instructions/msg.rs
/*
    use anchor_lang::prelude::*;

    #[derive(Accounts)]
    pub struct Msg<'info> {
        /// CHECK: no security
        pub dummy_acc: UncheckedAccount<'info>,
    }

    pub fn handler(ctx: Context<Msg>) -> Result<()> {
        msg!("You sent: {:?}", ctx.accounts.dummy_acc.key());
        Ok(())
    }
*/