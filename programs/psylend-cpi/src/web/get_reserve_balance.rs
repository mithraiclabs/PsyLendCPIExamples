// A basic example of reading a reserve's total balance (amount supplied and borrowed, in native
// decimals of the asset)

// To get the most up-to-date information, always send a refresh and accrue ix (or CPI) just prior
// to fetching this information. In this example, we don't care if the cache is stale. We crank this
// ix regularly, so the inaccuracy due to a stale cache is usually much less than 1%

use crate::state::{
    get_market_from_bytes, get_reserve_from_bytes, utilization_rate, Market, MarketReserves,
    Reserve, ReserveInfo,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetReserveBalance<'info> {
    /// The market that the reserve is under, you probably fetched this with Tokio
    /// CHECK: no checks
    #[account()]
    pub market: AccountInfo<'info>,
    /// A reserve where we want to see the current supply interest rate, you probably fetched this
    /// with Tokio.
    /// CHECK: no checks
    #[account()]
    pub reserve: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetReserveBalance>) -> Result<()> {
    // Markets, reserves, and obligations are all Plain Old Data (POD), and can be read from bytes
    // Don't forget to remove the 8-byte anchor discriminator...
    let market_data = &ctx.accounts.market.try_borrow_data()?[..][8..];
    let market: &Market = get_market_from_bytes(market_data);
    let reserve_data = &ctx.accounts.reserve.try_borrow_data()?[..][8..];
    let reserve: &Reserve = get_reserve_from_bytes(reserve_data);

    // This is the total amount of non-borrowed token, in native decimals, which is the same as the tvl
    let vault_total = reserve.total_deposits();
    msg!("total asset tvl, in native decimals: {:?}", vault_total);

    // The outstanding debt is all tokens loaned out to borrowers, plus interest/fees.
    // Stored as a Number, which is 10^15 decimals on top of native decimals.
    // E.g. 1 token is stored as 1 * 10 ^ decimals * 10 ^ 15
    let outstanding_debt = reserve.unwrap_outstanding_debt_unsafe();
    // Use to get native decimals, or `as_u64_rounded` to round to the nearest 1.
    let _outstanding_debt_native = outstanding_debt.as_u64(reserve.exponent);
    // For the most up-to-date information, run accrue_interest on the reserve first, then use:
    // let outstanding_debt = reserve.unwrap_outstanding_debt();

    // Rates are stored in Number format, using 10^15 decimals
    // e.g., 16% will be stored as 160,000,000,000,000
    // and 104% would be stored as 1,040,000,000,000,000
    let utilization_rate = utilization_rate(*outstanding_debt, vault_total);
    let interest_rate = reserve.interest_rate(*outstanding_debt, vault_total);

    msg!(
        "The utilization rate is {:?} and interest rate is: {:?}",
        interest_rate,
        utilization_rate
    );

    // Some additional info about reserves is cached on the market...
    let market_reserves: &MarketReserves = market.reserves();

    // If you want to see the circulating deposit/loan note supply, it can be read from the reserve
    let _deposit_note_mint_supply = reserve.total_deposit_notes();
    let _loan_note_mint_supply = reserve.total_loan_notes();

    // Exchange rates are stored on the market's cache for that reserve, allowing conversion of
    // notes into the asset. Note that due to looping deposits, the value of deposit notes may be
    // much higher than the actual value of deposited tokens.

    // E.g. 100 tokens deposited, but deposit notes may have a value of 300 tokens and loan notes
    // may be worth 250 tokens if:
    /*
       User deposits 100
       Borrows 100
       Deposits 100
       Borrows 100
       Deposits 100
       Borrows 50
       Total deposited: 350, Total borrowed: 250, Actual tvl: 50
    */
    let reserve_info: &ReserveInfo = market_reserves.get(reserve.index);
    let _deposit_note_exchange_rate = reserve_info
        .cached_note
        .get_stale()
        .deposit_note_exchange_rate;

    Ok(())
}
