use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use psy_math::Number;

use super::{Cache, FixedBuf, StoredPubkey, CACHE_TTL_LONG, ReserveIndex};

/// Lending market account. All reserves and obligations must fall under a market. Some
/// borrow-lending protocols prefer to use the terminology "pool"
///
/// Currently, there is a single primary market on mainnet (see constants.rs for the address)
///
/// Size = 16544 (plus 8 byte anchor discriminator)
#[account(zero_copy)]
pub struct Market {
    pub version: u32,

    /// UNUSED: The exponent used for quote prices
    pub quote_exponent: i32,

    /// The currency used for quote prices
    pub quote_currency: [u8; 15],

    /// The bump seed value for generating the authority address.
    pub authority_bump_seed: [u8; 1],

    /// The address used as the seed for generating the market authority
    /// address. Typically this is the market account's own address.
    pub authority_seed: Pubkey,

    /// The account derived by the program, which has authority over all
    /// assets in the market.
    pub market_authority: Pubkey,

    /// The account that has authority to make changes to the market
    pub owner: Pubkey,

    /// The mint for the token used by DEX to quote the value for reserve assets.
    pub quote_token_mint: Pubkey,

    /// Storage for flags that can be set on the market.
    pub flags: u64,

    /// State of rewards for the market.
    pub market_reward_state: MarketRewardState,

    /// Unused space before start of reserve list
    _reserved: [u8; 352],

    /// The storage for information on reserves in the market
    reserves: [u8; 15872],
}
impl Market {
    pub fn reserves(&self) -> &MarketReserves {
        bytemuck::from_bytes(&self.reserves)
    }
}

pub fn get_market_from_bytes(v: &[u8]) -> &Market{
    bytemuck::from_bytes(v)
}

/// Defines various paramaters for rewards. Some values do not change after initialization, others
/// can be updated between reward periods as needed.
///
/// Size = 160
#[derive(Pod, Zeroable, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
#[repr(C)]
pub struct MarketRewardState {
    /// Pubkey of MarketReward account.
    pub market_reward: Pubkey,

    /// Timestamp at which first reward distribution period index begins.
    /// The first reward period ends at `initial_reward_index_timestamp` + `distribution_period`
    /// This should not change after initialization.
    pub initial_reward_index_timestamp: i64,

    /// Length of each distribution period in seconds.
    /// This should not change after initialization.
    pub distribution_period: u64,

    /// Reward points allocated across whole market for each distribution period.
    pub reward_points_per_period: u64,

    /// Sum of reward multipliers across all Reserves in the market on each ObligationSide.
    pub total_reward_multiplier: u64,

    /// Time in seconds from start of period, after which withdrawal of rewards is allowed.
    /// Typically this is earlier than the end of the period.
    pub min_withdrawal_duration: u64,

    /// Number of decimal places for cumulative reward units.
    pub reward_unit_decimals: u8,

    /// Unused space
    pub _reserved0: [u8; 23],
    pub _reserved1: [u8; 64],
}

/// All reserves under the market cache exchange rate information and various key parameters on the market
/// 
/// Size = 15872
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct MarketReserves {
    /// Tracks the current prices of the tokens in reserve accounts
    reserve_info: [ReserveInfo; 32],
}
impl MarketReserves {
    pub fn get(&self, index: ReserveIndex) -> &ReserveInfo {
        &self.reserve_info[index as usize]
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &ReserveInfo> {
        self.reserve_info
            .iter()
            .take_while(|r| r.reserve != Pubkey::default())
    }
}

/// Stores cached data for a reserve
/// 
/// Size = 496
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ReserveInfo {
    /// The related reserve account
    pub reserve: StoredPubkey,

    /// Unused space
    _reserved: FixedBuf<80>,

    /// Cached values for the portion of the reserve info that is calculated dynamically
    pub cached_info: Cache<CachedReserveInfo, 1>,

    /// Cached values for reserve note exchange rates.
    pub cached_note: Cache<CachedNoteInfo, { CACHE_TTL_LONG }>,
}

/// Stores information about the reserve's exchange rates
/// 
/// Size = 176
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct CachedNoteInfo {
    /// The value of the deposit note (unit: reserve tokens per note token)
    pub deposit_note_exchange_rate: Number,

    /// The value of the loan note (unit: reserve tokens per note token)
    pub loan_note_exchange_rate: Number,

    /// Unused space
    _reserved: FixedBuf<128>,
}

/// Stores information about the reserve's config
/// 
/// Size = 176
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct CachedReserveInfo {
    /// The price of the asset being stored in the reserve account.
    /// USD per smallest unit (1u64) of a token
    pub price: Number,

    /// The minimum allowable collateralization ratio for a loan on this reserve
    pub min_collateral_ratio: Number,

    /// The bonus awarded to liquidators when repaying a loan in exchange for a
    /// collateral asset.
    pub liquidation_bonus: u16,

    /// Discount rate applied to tokens on this reserve (in bps)
    pub discount_rate: u16,

    /// Unused space
    _reserved: FixedBuf<124>,
}
