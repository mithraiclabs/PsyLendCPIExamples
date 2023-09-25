use crate::state::FixedBuf;
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use psy_math::Number;

use super::{Cache, CACHE_TTL_LONG, interpolate};

/// There are this many reward states/periods before the reward functionality becomes inoperable.
/// This is the number of indexes the array of states in the `market_rewards` struct can support.
pub const MAX_RESERVE_STATES: usize = 96;

// Size = 12
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct PsyFiVaultConfig {
    pub vault_account: Pubkey,
    pub collateral_token_decimals: u8,
    pub _reserved1: [u8; 31],
    pub _reserved2: [u8; 64],
}

/// All rates here are stored with a common denom of 10,000 (BPS_EXPONENT).
/// For example, a 5% rate would be stored as .05 * 10,000 = 500
/// 
/// We have three interest rate regimes. The rate is described by a continuous,
/// piecewise-linear function of the utilization rate:
/// 1. zero to [utilization_rate_1]: borrow rate increases linearly from
///     [borrow_rate_0] to [borrow_rate_1].
/// 2. [utilization_rate_1] to [utilization_rate_2]: borrow rate increases linearly
///     from [borrow_rate_1] to [borrow_rate_2].
/// 3. [utilization_rate_2] to one: borrow rate increases linearly from
///     [borrow_rate_2] to [borrow_rate_3].
///
/// Interest rates are nominal annual amounts, compounded continuously with
/// a day-count convention of actual-over-365. The accrual period is determined
/// by counting slots, and comparing against the number of slots per year.
/// Size = 64
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct ReserveConfig {
    /// The utilization rate at which we switch from the first to second regime.
    pub utilization_rate_1: u16,

    /// The utilization rate at which we switch from the second to third regime.
    pub utilization_rate_2: u16,

    /// The lowest borrow rate in the first regime. Essentially the minimum
    /// borrow rate possible for the reserve.
    pub borrow_rate_0: u16,

    /// The borrow rate at the transition point from the first to second regime.
    pub borrow_rate_1: u16,

    /// The borrow rate at the transition point from the second to thirs regime.
    pub borrow_rate_2: u16,

    /// The highest borrow rate in the third regime. Essentially the maximum
    /// borrow rate possible for the reserve.
    pub borrow_rate_3: u16,

    /// The minimum allowable collateralization ratio for an obligation
    pub min_collateral_ratio: u16,

    /// The amount given as a bonus to a liquidator
    pub liquidation_premium: u16,

    /// The threshold at which to collect the fees accumulated from interest into
    /// real deposit notes.
    pub manage_fee_collection_threshold: u64,

    /// The fee rate applied to the interest payments collected
    pub manage_fee_rate: u16,

    /// The fee rate applied as interest owed on new loans
    pub loan_origination_fee: u16,

    /// unused
    pub _reserved0: u16,

    /// Represented as a percentage of the Price
    /// confidence values above this will not be accepted
    pub confidence_threshold: u16,

    /// The maximum token amount to allow in a single DEX trade when
    /// liquidating assetr from this reserve as collateral.
    pub liquidation_dex_trade_max: u64,

    /// Multiplier that determines the fraction of reward points (by dividing over
    /// sum of all multiplier for the market) allocated for deposit notes.
    pub deposit_reward_multiplier: u8,

    /// Multiplier that determines the fraction of reward points (by dividing over
    /// sum of all multiplier for the market) allocated for loan notes.
    pub borrow_reward_multiplier: u8,

    pub _reserved1: [u8; 22],
}

/// A reserve tracks a single asset for depositing and/or lending, under a single market.
///
/// Size = 5184 (plus 8 byte anchor discriminator)
#[account(zero_copy)]
pub struct Reserve {
    pub version: u16,

    /// The unique id for this reserve within the market.
    /// Note: Should correspond to index of the `reserve_info` Vec on the market
    pub index: u16,

    /// The base 10 decimals used for token values
    /// Note: Typically stored as a negative to reflect the Pyth value, but the absolute value is
    /// used everywhere, so a positive number of equal magnitude can be used.
    pub exponent: i32,

    /// The market this reserve is a part of.
    pub market: Pubkey,

    /// The account where a Pyth oracle keeps the updated price of the token.
    pub pyth_oracle_price: Pubkey,

    /// The account where a Pyth oracle keeps metadata about the token.
    pub pyth_oracle_product: Pubkey,

    /// The mint for the token being held in this reserve
    pub token_mint: Pubkey,

    /// The mint for this reserve's deposit notes. Uses `token_mint` decimals.
    pub deposit_note_mint: Pubkey,

    /// The mint for this reserve's loan notes. Uses `token_mint` decimals.
    pub loan_note_mint: Pubkey,

    /// The account with custody over the reserve's tokens.
    pub vault: Pubkey,

    /// The account with custody of the notes generated from collected fees
    pub fee_note_vault: Pubkey,

    /// The account for storing quote tokens during a swap
    pub dex_swap_tokens: Pubkey,

    /// The account used for trading with the DEX
    pub dex_open_orders: Pubkey,

    /// The DEX market account that this reserve can trade in
    pub dex_market: Pubkey,

    pub _reserved0: [u8; 408],

    pub config: ReserveConfig,

    pub psyfi_vault_config: PsyFiVaultConfig,

    /// Discount rate for the token this reserve uses, updated from the common discounts account
    pub discount_rate: u16,

    /// Current version of the discount rate. If lower than the version in the discounts account,
    /// should be updated.
    pub discount_rate_version: u16,

    /// Indicates if the reserve has halted borrows, repays, or deposits:
    ///
    /// 0 (0b00000000) = nothing halted,
    /// 1 (0b00000001) = deposits halted,
    /// 2 (0b00000010) = borrows halted,
    /// 4 (0b00000100) = repays halted,
    /// 8 (0b00001000) = withdraws halted
    ///
    /// Allows addition or bitwise AND to combine multiple states:
    /// (e.g., 4 + 2 = 6 (0b00000110) = borrows and repays halted, others allowed)
    pub halt_state: u8,

    _reserved1: [u8; 123],

    // Define a u128 array here so struct aligns as 16 bytes for fields in ReserveState.
    _reserved2: [u128; 32],

    state: [u8; 3584],
}

pub fn get_reserve_from_bytes(v: &[u8]) -> &Reserve{
    bytemuck::from_bytes(v)
}

impl Reserve {
    fn state(&self) -> &Cache<ReserveState, { CACHE_TTL_LONG }> {
        bytemuck::from_bytes(&self.state)
    }
    /// Get state, fail if expired. Accrue first, or this will fail
    fn unwrap_state(&self, current_slot: u64) -> &ReserveState {
        self.state()
            .expect(current_slot, "Reserve needs to be refreshed")
    }
    /// Get state regardless of age. State may be any arbitrary number of slots old.
    fn state_stale(&self) -> &ReserveState {
        self.state().get_stale()
    }

    // The following values are always safe to extract from stale state, because any
    // withdraw/deposit/borrow requires an accrue already.
    /// Total deposited tokens that are not currently borrowed, in token. Safe to extract from stale
    /// state, because any withdraw/deposit/borrow requires an accrue.
    pub fn total_deposits(&self) -> u64 {
        self.state_stale().total_deposits
    }
    pub fn total_deposit_notes(&self) -> u64 {
        self.state_stale().total_deposit_notes
    }
    pub fn total_loan_notes(&self) -> u64 {
        self.state_stale().total_loan_notes
    }
    pub fn accrued_until(&self) -> i64 {
        self.state_stale().accrued_until
    }

    /// Example to see actual current oustanding debt, which may update over time due to interest
    pub fn unwrap_outstanding_debt(&self, current_slot: u64) -> &Number {
        &self.unwrap_state(current_slot).outstanding_debt
    }
    /// For more approximate interest calculations, may be inaccurate unless accrue has occured recently.
    pub fn unwrap_outstanding_debt_unsafe(&self) -> &Number {
        &self.state_stale().outstanding_debt
    }

    /// Get the current interest rate for this reserve
    pub fn interest_rate(&self, outstanding_debt: Number, vault_total: u64) -> Number {
        let borrow_1 = Number::from_bps(self.config.borrow_rate_1);

        // Catch the edge case of empty reserve
        if vault_total == 0 && outstanding_debt == Number::ZERO {
            return borrow_1;
        }

        let util_rate = utilization_rate(outstanding_debt, vault_total);

        let util_1 = Number::from_bps(self.config.utilization_rate_1);

        if util_rate <= util_1 {
            // First regime
            let borrow_0 = Number::from_bps(self.config.borrow_rate_0);

            return interpolate(util_rate, Number::ZERO, util_1, borrow_0, borrow_1);
        }

        let util_2 = Number::from_bps(self.config.utilization_rate_2);
        let borrow_2 = Number::from_bps(self.config.borrow_rate_2);

        if util_rate <= util_2 {
            // Second regime
            let borrow_1 = Number::from_bps(self.config.borrow_rate_1);

            return interpolate(util_rate, util_1, util_2, borrow_1, borrow_2);
        }

        let borrow_3 = Number::from_bps(self.config.borrow_rate_3);

        if util_rate < Number::ONE {
            // Third regime
            return interpolate(util_rate, util_2, Number::ONE, borrow_2, borrow_3);
        }

        // Maximum interest
        borrow_3
    }
}

/// Get the current utilization rate (borrowed / deposited)
pub fn utilization_rate(outstanding_debt: Number, vault_total: u64) -> Number {
    outstanding_debt / (outstanding_debt + Number::from(vault_total))
}

/// Information about a reserve's current assets and outstanding deposit/loan notes and reward units.
///
/// Notes:
/// To get the total amount of tokens under the reserve's control, use:
/// (`total_deposits` + `outstanding_debt` - `uncollected_fees`)
///
/// For total deposit/loan notes in circulation: `total_deposit_notes`, `total_loan_notes`
/// Size = 3568
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct ReserveState {
    accrued_until: i64,

    /// Amount of deposited tokens loaned out to borrowers, plus amount of tokens charged
    /// in fees (in U192 Number, in token.)
    ///
    /// Storage format example: debt is 5.927 token, with 8 token decimals
    /// `5.927 * 10 ^ 8 * 10 ^ 15 = 592,700,000,000,000,000,000,000`
    outstanding_debt: Number,

    /// Amount of fees collected (in U192 Number, in token). `Outstanding debt` includes
    /// this amount.
    ///
    /// Storage format example: fees are 5.927 token, with 8 token decimals
    /// `5.927 * 10 ^ 8 * 10 ^ 15 = 592,700,000,000,000,000,000,000`
    uncollected_fees: Number,

    /// Amount of deposited tokens that is not borrowed (in token)
    total_deposits: u64,

    /// Amount of deposit notes issued and in circulation (i.e., matches mint's supply)
    total_deposit_notes: u64,

    /// Amount of loan notes issued and in circulation (i.e., matches mint's supply)
    total_loan_notes: u64,

    /// Each index corresponds to cummulative sum of the reward points distributed
    /// per deposit note for the distribution period. This value is denominated in
    /// reward_unit_decimals.
    cummulative_deposit_reward_units: [u128; MAX_RESERVE_STATES],

    /// Each index corresponds to cummulative sum of the reward points distributed
    /// distributed per loan note for the distribution period. This value is denominated in
    /// reward_unit_decimals.
    cummulative_loan_reward_units: [u128; MAX_RESERVE_STATES],

    _reserved: FixedBuf<416>,
}
