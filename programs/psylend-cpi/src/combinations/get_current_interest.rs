// A basic example of reading the current reserve interest rate, note/token exchange rate, and other
// important information from the chain.

// To get the most up-to-date information, always send a refresh and accrue ix (or CPI) just prior
// to fetching this information. In this example, we don't care if the cache is stale.

use crate::state::{
    get_market_from_bytes, get_reserve_from_bytes, utilization_rate, Market, MarketReserves,
    Reserve, ReserveInfo,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetCurrentInterest<'info> {
    /// The market that the reserve is under
    /// CHECK: no checks
    #[account()]
    pub market: AccountInfo<'info>,
    /// A reserve where we want to see the current supply interest rate
    /// CHECK: no checks
    #[account()]
    pub reserve: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetCurrentInterest>) -> Result<()> {
    // Markets, reserves, and obligations are all Plain Old Data (POD), and can be read from bytes
    // Don't forget to remove the 8-byte anchor discriminator...
    let market_data = &ctx.accounts.market.try_borrow_data()?[..][8..];
    let market: &Market = get_market_from_bytes(market_data);
    let reserve_data = &ctx.accounts.reserve.try_borrow_data()?[..][8..];
    let reserve: &Reserve = get_reserve_from_bytes(reserve_data);

    let market_reserves: &MarketReserves = market.reserves();
    let _reserve_info: &ReserveInfo = market_reserves.get(reserve.index);

    let vault_total = reserve.total_deposits();
    let _deposit_note_mint_supply = reserve.total_deposit_notes();
    let _loan_note_mint_supply = reserve.total_loan_notes();

    let outstanding_debt = reserve.unwrap_outstanding_debt_unsafe();
    // For the most up-to-date information, run accrue_interest on the reserve first, then use:
    // let outstanding_debt = reserve.unwrap_outstanding_debt();

    // rates are stored in Number format, which is 10^15 decimals
    // e.g., 16% will be stored as 160,000,000,000,000
    // and 104% would be stored as 1,040,000,000,000,000
    let utilization_rate = utilization_rate(*outstanding_debt, vault_total);
    let interest_rate = reserve.interest_rate(*outstanding_debt, vault_total);

    msg!(
        "The utilization rate is {:?} and interest rate is: {:?}",
        interest_rate,
        utilization_rate
    );

    Ok(())
}
