use crate::state::FixedBuf;
use anchor_lang::prelude::*;
use bytemuck::{Contiguous, Pod, Zeroable};
use psy_math::Number;

use super::{StoredPubkey, MAX_RESERVE_STATES};

/// Describes the technical maximum number of supported positions. Loan/obligations arrays are 2560
/// bytes and each position = 160 bytes. The max is 2560/160 = 16
///
/// PsyLend uses `MAX_ALLOWED_POSITIONS` to enforce a maximum below this value (to avoid
/// limitations on transaction size using regular transactions)
const _MAX_POSITIONS_TECHNICAL: usize = 16;

/// If true, ixes that open a new position on the obligation (borrow, deposit collateral), will fail
/// once the `MAX_ALLOWED_POSITIONS` limit is reached. If false, will allow positions to be opened
/// indefinitely, though transaction size limits will eventually cause certain ixes to fail, as
/// every reserve used on the obligation must be refreshed within the same slot.
///
/// With versioned transactions, obligations can now support more positions, up to the
/// technical maximum. PsyLend may eventually disable this restriction.
pub const _CAP_ALLOWED_POSITIONS: bool = true;

/// Once this many collateral + loan positions are opened on an obligation, prevents borrowing or
/// depositing on additional reserves. Only used if `CAP_ALLOWED_POSITIONS` is enabled.
pub const _MAX_ALLOWED_POSITIONS: usize = 6;

// 4 + 4 + 32 + 32 + 184 + 256 + 2560 + 2560 + (16 * 96)
// Size = 7168 (plus 8 byte anchor discriminator)
/// Tracks information about a user's obligation to repay a borrowed position.
#[account(zero_copy)]
pub struct Obligation {
    pub version: u32,

    pub _reserved0: u32,

    /// The market this obligation is a part of
    pub market: Pubkey,

    /// The address that owns the debt/assets as a part of this obligation
    pub owner: Pubkey,

    /// Unused space before start of collateral info
    pub _reserved1: [u8; 184],

    /// Storage for cached calculations
    pub cached: [u8; 256],

    /// Storage for information on collateral positions owned by this obligation. (See `ObligationSide`)
    pub collateral: [u8; 2560],

    /// Storage for information on loan positions owned by this obligation. (See `ObligationSide`)
    pub loans: [u8; 2560],

    /// Each index corresponds to amount of reward units accrued during a
    /// sequential distribution period, that is not yet claimed.
    /// This value is denominated in reward_unit_decimals.
    pub accrued_reward_units: [u128; MAX_RESERVE_STATES],
}

pub fn get_obligation_from_bytes(v: &[u8]) -> &Obligation {
    bytemuck::from_bytes(v)
}

// Size 2560
/// Tracks information about the collateral/loans on an obligation
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ObligationSide {
    positions: [Position; 16],
}

impl ObligationSide {
    /// Get position or panic
    pub fn position(&self, account: &Pubkey) -> Option<&Position> {
        self.positions.iter().find(|p| p.account.eq(account))
    }

    pub fn position_with_index(&self, index: &ReserveIndex) -> Option<&Position> {
        self.positions.iter().find(|p| p.reserve_index.eq(index))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Position> {
        self.positions
            .iter()
            .filter(|p| p.account != Pubkey::default())
    }
}

// Size = 160
/// Information about a single collateral or loan account registered with an obligation
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
#[repr(C)]
pub struct Position {
    /// The token account holding the bank notes
    pub account: StoredPubkey,

    /// Non-authoritative number of bank notes placed in the account
    pub amount: Number,

    /// Timestamp when Position was last changed, used to determine time to start accruing rewards from.
    /// Has to be initialized to System timestamp on Position creation.
    pub last_updated: i64,

    /// Cummulative sum of the reward points distributed per corresponding deposit or
    /// loan note from start of the last_updated period to last_updated timestamp.
    /// This value is denominated in reward_unit_decimals, for each unit of deposit or loan
    /// note.
    /// Has to be initialized to cummulative rewards units at time of Position creation.
    pub cummulative_reward_units: u128,

    pub side: u32,

    /// The index of the reserve that this position's assets are from
    pub reserve_index: ReserveIndex,

    _reserved: FixedBuf<74>,
}

/// Collateral or Loan
#[derive(Contiguous, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum Side {
    Collateral = 0,
    Loan = 1,
}

pub type ReserveIndex = u16;

impl Obligation {
    /// Determine if this obligation has a custody over some account,
    /// by checking if its in the list of registered accounts.
    pub fn has_collateral_custody(&self, account: &Pubkey) -> bool {
        self.collateral()
            .iter()
            .any(|p| p.account.as_ref() == account)
    }

    /// Determine if this obligation has a custody over some account,
    /// by checking if its in the list of registered accounts.
    pub fn has_loan_custody(&self, account: &Pubkey) -> bool {
        self.loans().iter().any(|p| p.account.as_ref() == account)
    }

    pub fn position_count(&self) -> usize {
        self.collateral_positions_count() + self.loan_positions_count()
    }

    pub fn collateral_positions_count(&self) -> usize {
        self.collateral().iter().count()
    }

    pub fn loan_positions_count(&self) -> usize {
        self.loans().iter().count()
    }

    pub fn collateral(&self) -> &ObligationSide {
        bytemuck::from_bytes(&self.collateral)
    }

    pub fn loans(&self) -> &ObligationSide {
        bytemuck::from_bytes(&self.loans)
    }
}
