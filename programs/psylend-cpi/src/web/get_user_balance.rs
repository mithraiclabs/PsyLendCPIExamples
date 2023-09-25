// A basic example of reading a a user's balances (amount supplied and borrowed, in native decimals
// of the asset)

// To get the most up-to-date information, always send a refresh and accrue ix (or CPI) just prior
// to fetching this information. In this example, we don't care if the cache is stale. We crank this
// ix regularly, so the inaccuracy due to a stale cache is usually much less than 1%

use crate::state::{
    get_market_from_bytes, get_obligation_from_bytes, get_reserve_from_bytes,
    Market, MarketReserves, Obligation, Reserve, ReserveInfo,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetUserBalance<'info> {
    /// The market that the reserve is under
    /// CHECK: no checks
    #[account()]
    pub market: AccountInfo<'info>,
    /// A reserve where we want to see a user's balance
    /// CHECK: no checks
    #[account()]
    pub reserve: AccountInfo<'info>,
    /// An obligation for user whose balances we want to read.
    /// Obligation seeds are: `b"obligation".as_ref(), market.key().as_ref(), user.key.as_ref()`
    /// CHECK: no checks
    #[account()]
    pub user_obligation: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetUserBalance>) -> Result<()> {
    // Markets, reserves, and obligations are all Plain Old Data (POD), and can be read from bytes
    // Don't forget to remove the 8-byte anchor discriminator...
    let market_data = &ctx.accounts.market.try_borrow_data()?[..][8..];
    let market: &Market = get_market_from_bytes(market_data);
    let reserve_data = &ctx.accounts.reserve.try_borrow_data()?[..][8..];
    let reserve: &Reserve = get_reserve_from_bytes(reserve_data);
    let obligation_data = &ctx.accounts.user_obligation.try_borrow_data()?[..][8..];
    let obligation: &Obligation = get_obligation_from_bytes(obligation_data);

    // Read the user's deposited tokens on some reserve (in deposit notes)
    let user_deposit_position = obligation.collateral().position_with_index(&reserve.index);
    if user_deposit_position.is_some() {
        msg!(
            "The user's balance on this position in deposit notes is: {:?}",
            user_deposit_position.unwrap().amount
        )
    }

    // Convert deposit notes to actual token
    let market_reserves: &MarketReserves = market.reserves();
    let reserve_info: &ReserveInfo = market_reserves.get(reserve.index);
    // For the must up-to-date exchange rate, run `accrue_interest` on the reserve first.
    let deposit_note_exchange_rate = reserve_info
        .cached_note
        .get_stale()
        .deposit_note_exchange_rate;
    if user_deposit_position.is_some() {
        let user_deposit_native = (user_deposit_position.unwrap().amount
            * deposit_note_exchange_rate)
            .as_u64_rounded(reserve.exponent);
        // Note: You may want .as_u64_rounded(0); instead to keep native decimals
        msg!(
            "The user's balance on this position in native token is: {:?}",
            user_deposit_native
        )
    }

    // Read the user's borrowed tokens on some reserve (in loan notes)
    let user_loan_position = obligation.loans().position_with_index(&reserve.index);
    if user_loan_position.is_some() {
        msg!(
            "The user's balance on this position in loan notes is: {:?}",
            user_loan_position.unwrap().amount
        )
    }

    // Convert loan notes to actual token
    let loan_note_exchange_rate = reserve_info.cached_note.get_stale().loan_note_exchange_rate;
    if user_loan_position.is_some() {
        let user_loan_native = (user_loan_position.unwrap().amount * loan_note_exchange_rate)
            .as_u64_rounded(reserve.exponent);
        // Note: You may want .as_u64_rounded(0); instead to keep native decimals
        msg!(
            "The user's borrowed amount on this position in native token is: {:?}",
            user_loan_native
        )
    }

    Ok(())
}
