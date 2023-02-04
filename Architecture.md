<h1>Architecture Documentation and Instruction List</h1>

## Common Terms
* Key - The public key, unless otherwise specified
* Signer - User authorizing the transaction
* Oracle - An on-chain address that reports the current price of an asset
* Decimals - All currencies use whole integers. To represent decimal numbers, some number of the
  rightmost digits of integers are used as decimals. We call this number of digits the `decimals`.
  For example, 123.45 is expressed as 12345 in a system with 2 decimals. Note that converting back
  to the original number would require division by 10^2.
* Position - A collateral or borrowed position, i.e. some tokens that have been put up as collateral 
  or borrowed by a user
* C Ratio - The Collateral Ratio is defined as the sum of a user's collateral divided by the sum of
  their debts. It is the threshold for liquidation: an account with a C-Ratio below the minimum is
  eligible to be liquidated. 
* TTL - All cached data has a certain Time-To-Live, expressed in slots. Once TTL slots have elapsed,
  instructions that attempt to use that cache without refreshing it will fail. The time of one slot
  varies depending on the performance of the Solana network, but is generally about 400ms.
* PsyFi - The PsyFi-euros program. In short, this program creates vaults which sell options on some
  asset. Users deposit funds into the vault, and get vault tokens (more specifically called `vault
  ownership tokens`) as proof of deposit. PsyLend enables users to use vault tokens as collateral.
* Common Denom - All percentages, such as fees, interest rates, etc, use a common denominator of
  10,000. E.g. when passing 5%, one would pass 10,000 * .05 = 500
* Reward Point - one whole denomination of a reward. A reward point represents an entitlement to
  some portion of the reward pool, which may be some combination of whichever SPL tokens are offered
  as rewards at that time. 
* Reward Unit - the smallest fraction of a reward point, as defined by the number of reward unit decimals.

## Important States and Structs
* Reserve - A Reserve tracks an asset that can be borrowed or used as collateral. The Reserve stores
  information about interest rates, fees, note exchange rates, liquidation premium, discount rates,
  and more. The Reserve records tokens deposited and borrowed, as well as interest earned.
* PsyFi Reserve - A Reserve that tracks a PsyFi asset (a vault token). Vault tokens are valued based
  on the price of the underlying AND the option that the vault sells, meaning vault tokens do NOT
  have a one to one relationship with the underlying asset they represent.
* Jet Market - All Reserves belong to a Jet Market. This structure caches information from each
  Reserve, enabling various instructions to see information about any Reserve without needing to pass
  an additional account.
* Cache - There are two primary caches: `CachedNoteInfo` aka NOTE and `CachedReserveInfo` aka INFO. 
    * The NOTE cache tracks information about note exchange rates (which update mainly as interest
    accrues), and has a flexible TTL (usually within several slots). 
    * The INFO cache tracks price, 
    C Ratio, liquidation
    bonus, and discount rate, and must be refreshed within the same slot it is used. 
    * The Obligation
    also utilizes a `CalculationCache` to store results of common math operations to reduce compute
    overhead.  
    * The `ReserveState` aka DATA cache tracks reserve-specific information about interest,
    fees, deposits, and outstanding notes. It is conceptually closely tied to the NOTE cache, and uses 
    the same TTL.
* Obligation - Each user has one obligation per market. The Obligation tracks all of a users
  collateral and borrowed positions, and caches various math related to the value of the positions.
  The obligation is a PDA derived per user per market.
* Discounts - Some currencies are treated below their face value for collateral purposes, this is
  called a discount rate. The discounts struct is per-program, and tracks discount rates for
  reserves in all markets. If a reserve asset is not present, it uses the default rate (the face value).
* MarketReward - Contains information on reward distribution state for a single Market. MarketReward
contains an array of 96 `RewardState` that corresponds to each reward distribution period. Index 0
is the first (i.e. oldest) `RewardState`. Each `RewardState` can store up to 5 `RewardInfo`, which
corresponds to reward tokens being issued in that period. Other important information about rewards:
  * All timestamps and durations use seconds.
  * Initial Reward Index Timestamp (on Market) - Rewards begin to accumulate after this time, which must be in
    the future when rewards are first initialized.
  * Distribution Period (on Market) - A minimum of one week. Cannot change once a `MarketReward` is initialized.
    The first `RewardState` expires this amount of time after the `Initial Reward Index Timestamp`.
    Each subsequent `RewardState` expires the same amount of time after its predecessor.
  * Min Withdrawal Duration (on Market) - The first `RewardState` becomes eligible for
    withdrawal this amount of time after the `initial_reward_index_timestamp`. Each subsequent
    `RewardState` becomes eligible to withdraw this amount of time after it begins. This value
    can be updated, but once a period has been initialized, it is locked in for that period, even if
    this value (stored on the market) is updated. The cannonical withdraw time for a given
    `RewardState` should be read directly from that `RewardState`.
  * Unused `RewardState` and `RewardInfo` are initialized as zeroes, accounts will have the default Pubkey.
* Client Accounts - For each reserve, users need a deposits, loan, and collateral account. 
    * The deposits account stores deposit notes that have been deposited with the program but not used as
    collateral, 
    * the loan account stores the balance of loan notes, 
    * the collateral account stores the balance of deposit notes used as collateral. 

  These accounts are all PDAs derived per user per reserve, and the Jet Market acts as the token
  acount authority.
* Reserve Accounts - Each reserve has the following derived accounts: 
    * vault - holds the reserve's actual token assets, not to be confused with PsyFi Vaults, 
    * feeNoteVault - holds fees earned, 
    * dexOpenOrders - the Serum Dex open orders account this reserve can trade under, 
    * dexSwapTokens - the Serum Dex swap account this reserve can trade under, 
    * loanNoteMint - mints loan notes to user loan accounts when users borrow funds 
    * depositNoteMint - mints deposit notes to user deposit accounts when users deposit funds. 

These accounts are all PDAs derived per reserve.
* PsyFi Vault - The top-level address of a PsyFi-euros struct that contains a variety of information
  about the token type, oracle address, balance, and more. Not to be confused with the `vault` of a
  Reserve, which is simply the token account where the Reserve holds assets. In this program, the
  PsyFi Vault is occasionally passed to functions that support PsyFi reserves to validate that the
  Reserve and PsyFi vault use the same currency, oracle, etc.
* Number - A U192 value, which uses 15 decimal places. Note that attempting to load a Number in TS/JS
  will fail due to trailing decimal places, load it as a BN and divide by 10e15 to get the whole
  number value, or use one the `jetBNToNumber` in `math.ts` (in package) to get the actual floating
  point value, or use one of the functions in `tools.ts` (in package) if the Number is in Buffer
  format (like when reading from Obligation directly).
* Amount - Stores one of three kinds of units (Token, Deposit Note, Loan Note), and a u64 value.
  Most instructions that require currency input use an Amount as input.

********************************************************************************


## Categories of Instruction

### Adminstration and Market Making:

init_discounts, init_market, init_market_reward, init_psyfi_reserve, init_reserve, set_market_flags,
set_market_owner, update_discounts, update_market_reward, update_reserve_config,
update_reserve_halts, update_reserve_reward

********************************************************************************
### User Executed

#### Initilization/Closing Accounts Only

close_collateral_account, close_deposit_account, cloan_loan_account, close_obligation,
deposit_collateral, init_collateral_account, init_deposit_account, init_loan_account,
init_obligation,

#### Regular Use

accrue_position_reward, borrow, claim_rewards, deposit_tokens/deposit, repay, withdraw_collateral, 
withdraw_tokens/withdraw

********************************************************************************
### Recurring or Scheduled, Anyone Can Execute (Permissionless)

accrue_interest, refresh_psyfi_reserve, refresh_reserve, sync_discount_rates,

********************************************************************************
### Special

liquidate_dex, liquidate




## Detailed Instruction Guide

********************************************************************************
## accrue_interest

### Description: 
Updates the NOTE and DATA cache for a reserve. Accrues interest on the reserve, which typically increases
loan balances, accumulates interest earned by depositors, updates fees, and updates note exchange
rates. This instruction also accrues reward points for both lending and borrowing for the Reserve.

Permissionless. The market adminstrator will crank this instruction periodically, but any user may
send it when an updated cache is required.

### Arguments:
None.

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Reserve
* FeeNoteVault - can be extracted from `reserve`
* DepositNoteMint - can be extracted from `reserve`
 
### Notes:
 
In some instances the accrue IX may not be able to finish in one go, and must be run multiple times
to fully accumulate interest and update the caches. The maximum amount of time that can elapse
before the accrue is not able to complete in one iteration is currently one week.

This IX consumes uncollected fees in the reserve, minting deposit notes to the
`fee_note_vault`. It also increases the loan debt (aka loan token supply) and thusly increases the 
deposit token supply (which counts all outstanding assets, including owed debt).

********************************************************************************
## accrue_position_reward

### Description: 
Used for accruing rewards for a user's `Position` in an `Obligation`.

### Arguments:
* side: 0 for collateral, 1 for loan
### Accounts:
* Market - jet market
* Reserve - reserve to modify 
* Obligation - obligation account
* Owner - owner of obligation
* PositionAccount - collateral or loan account for Obligation that matches the Reserve. 

### Notes:
This ix can be invoked directly, or will be invoked indirectly whenever a user performs an action that modifies their `Position` in an `Obligation`. These actions include: `deposit_collateral`, `withdraw_collateral`, `borrow`, `repay` and `liquidate`.

Rewards will be accrued from the last time accrual was done for the position, to the most recent reward accrual for the `Reserve`. These reward units earned is stored at the Obligation-level, as a sum of reward units earned per period for the user.

********************************************************************************
## borrow

### Description: 
Borrows funds against a user's collateral. Requires a general refresh of all reserves the user
has borrowed or used as collateral

### Arguments:
* Bump - for the user's loan account
* Amount - quantity to borrow, generally specified in tokens

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Reserve
* Obligation - user's obligation on this market, must be initialized
* Vault - can be extracted from `reserve`
* LoanNoteMint - can be extracted from `reserve`
* Borrower - aka UserAddress, typically the user's wallet, must sign tx.
* LoanAccount - user's loan account, must be initialized
* Receiver - borrowed tokens will be transferred here.
 
### Notes:
 
User loan and obligation account must have been initialized beforehand.

********************************************************************************
## claim_rewards

### Description: 
Used for claiming reward tokens in exchange for reward units stored in an `Obligation`.

### Arguments:
* period_to_claim: should be between 0 to 95.
### Accounts:
* Market - jet market
* MarketAuthority - authority of reward token account
* MarketReward 
* Obligation - obligation account
* Owner - owner of obligation
* TokenProgram
* RemainingAccounts - this should be a sequential pair of reward token account in `RewardInfo`
  followed by the owner's token account. Must match the order the token types appear in `RewardInfo`

### Notes:
Remaining accounts has to contain all of the reward token accounts in period to claim and the owner's token accounts, in the correct order.

Claiming of reward tokens is allowed after the `withdrawal_time` elapses. This time is
locked in when the period is initialized, and is the start of the period +
`min_withdrawal_duration`. The `min_withdrawal_duration` is stored on the Market, and may change
from period to period, but will not change once a period has been initialized.

In the case where there are insufficient claimable reward units remaining, no reward tokens will be issued for the excess reward units.

********************************************************************************
## close_collateral_account, close_deposit_account, cloan_loan_account, close_obligation,

### Description: 
Closes an account, removing it from the obligation. Primarily used to recover rent fees. Once all
loan/collateral accounts on an obligation are closed, the obligation can also be closed to recover rent.

### Arguments:
* None.

### Accounts:
* Varies
 
### Notes:

Closing an obligation will fail if collateral or loan accounts still exist on it, close those
accounts first before attempting to close the obligation.. The signer of the tx is required to own
the account they are trying to close. Lamports will go to the owner/signer.
 
********************************************************************************
## deposit, deposit_tokens

### Description: 
Deposits assets into the users deposit account. These assets cannot be used as collateral until they
are sent to the collateral account. Funds go in the vault, and deposit account is minted deposit notes.

Only the reserve being deposited into needs to be accrued, run the `accrue_interest` ix.

### Arguments:
* Bump - for the user's deposit account
* Amount - quantity to deposit, specified in tokens or deposit notes

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* DepositSource - tokens come from this account
* DepositAccount - Deposit notes will go to this account. For `deposit`, user's deposit account,
  which must be initialized. For `deposit_tokens`, may be any account. 
* Depositor - aka UserAddress, typically the user's wallet, must sign tx.
* Reserve
* Vault - can be extracted from `reserve`
* DepositNoteMint - can be extracted from `reserve`
 
### Notes:
 
These instructions are nearly identical: deposit calls deposit_tokens. The `deposit_tokens` ix does
not check that the `deposit_account` is a valid PDA of this program, enabling users to claim deposit
notes to an account of their choosing. A user may wish to do this in order to stake these notes or
trade them externally. 

Notes taken outside the program this way cannot be used as collateral unless
they are transferred back to a valid deposit account, because `deposit_collateral` requires notes to
be in a valid deposit account.

User deposit account must have been initialized beforehand.

********************************************************************************
## deposit_collateral

### Description: 
Transfers assets from the deposits account into the users collateral account. 
These assets can be borrowed against.

Only the reserve being deposited into needs to be accrued, run the `accrue_interest` ix.

### Arguments:
* Bump - for the user's collateral account
* Amount - quantity to transfer, specified in tokens or deposit notes

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Obligation - user's obligation on this market, must be initialized
* DepositAccount - user's deposit account, must be initialized
* CollateralAccount - user's collateral account, must be initialized
* Depositor - aka UserAddress, typically the user's wallet, must sign tx.
* Reserve
* DepositNoteMint - can be extracted from `reserve`
 
### Notes:
 
User deposit, collateral, and obligation accounts must have been initialized beforehand. User must
deposit some funds before calling this ix. Can be bundled in the same transaction as a deposit.

********************************************************************************
## init_collateral_account, init_deposit_account, init_loan_account, init_obligation

### Description: 
Before they can be used, user accounts need to be initialized. Derive the PDA and use the
appropriate ix to initialize the account before using it.

### Arguments:
* Bump - for deriving the account

### Accounts:
* Varies
 
### Notes:
 
A getOrInit utility for each of these accounts is available in Instructions. Obligation is
per-user per-market, while others are per-user per-reserve.

********************************************************************************
## init_discounts, init_market, init_psyfi_reserve, init_reserve

### Description: 
The market administrator uses these instructions to create new markets and reserves.

### Arguments:
* Varies

### Accounts:
* Varies
 
### Notes:

The administrator will create a market first, then any reserves, and then add discount rates as
desired. The discount rate is created once per program.

********************************************************************************
## init_market_reward

### Description: 
Used for initializing `MarketRewardState` in `Market` struct, that stores configuration on reward distribution rate and 
start timestamp, and `MarketReward` struct that contains that state of reward distribution.

This should be run once after initialization of `Market`.

### Arguments:
* initial_reward_index_timestamp - timestamp for start of first reward distribution period
* distribution_period - length in seconds of each distribution period
* reward_points_per_period - number of reward points to issue across entire market for each period
* reward_unit_decimals - number of decimals to represent each reward point in
* minWithdrawalDuration - min duration from start of period, after which rewards claim is allowed

### Accounts:
* Market - jet market
* Owner - owner of market
* MarketReward - market reward to be initialized
* SystemProgram

### Notes:
The MarketReward struct is a regular account, not a PDA, and must be created with the SystemProgram
before running this ix.

********************************************************************************
## liquidate_dex

### Description: 
Unused. This ix is currently disabled, calling it will trigger a panic.

### Arguments:
* NA

### Accounts:
* NA
 
### Notes:

None.
 
********************************************************************************
## liquidate

### Description: 
A liquidator calls this instruction on a user whose obligation is underwater to seize a portion of
their collateral and repay a portion of their loan.

### Arguments:
* Amount - quantity to repay, specified in tokens or loan notes
* MinCollateral - the minimum amount of collateral the liquidator will accept in exchange for the
  offered repayment. If a smaller amount of collateral is computed by the ix, the tx will fail. The
  instruction will compute the actual amount of collateral seized, which may be higher than this amount.
  Can supply 0 here to liquidate regardless of the collateral seized.

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Obligation - user's obligation on this market, must be initialized
* Reserve - the reserve with the DEBT borrowed
* CollateralReserve - reserve with COLLATERAL to seize
* Vault - can be extracted from `reserve` (holds the tokens the reserve uses)
* LoanNoteMint - mint for debt/loan notes on `reserve`
* LoanAccount - users's loan account on `reserve`, must be initialized
* CollateralAccount - user's collateral account on `collateralReserve`, must be initialized
* PayerAccount - The liquidator's token account for the asset being repaid (same token as the `vault`)
* ReceiverAccount - The liquidator's collateral account on `reserve`, which will recieve the seized notes.
* Payer - Signs the tx, typically the liquidator's wallet
 
### Notes:

A loan is underwater when the user falls below the minimum C Ratio, where the smallest C Ratio of
all reserves where the user has borrowed funds applies. Remember that the C Ratio is the sum of
all collateral, divided by the sum of all loans.
 
When liquidating an unhealthy account with many assets, liquidators can specify whichever loan
account they want to repay and whichever collateral account they want to seize.

Liquidators must set minCollateral appropriately to ensure that liquidation is profitable. When this
value is not used, liquidation can result in a loss.

Liquidation is only profitable when restoring an account to the min C Ratio, seizing more assets
will generally result in a loss to the liquidator.

********************************************************************************
## refresh_psyfi_reserve, refresh_reserve

### Description: 
Updates various reserve properties. Updates the INFO cache. Must be invoked first, in the same tx,
as most transactions that read the status of a reserve. Most transactions require a refresh ix for
every reserve that user is involved with (collateral or borrowed). This is sometimes called a
general refresh of all reserves.

Permissionless. This instruction is sent any time funds are updated (deposit, borrow, repay,
withdraw, etc), by the user engaging in that instruction. The market adminstrator does not generally
crank this instruction otherwise.

### Arguments:
None

### Accounts:
* Market - jet market the reserve is under
* Reserve
* (PsyFi only) VaultAccount - the vault where the vault tokens this reserve users come from
* pythOraclePrice - can be extracted from `reserve`
 
### Notes:
 
The VaultAccount is required only for PsyFi reserves.

********************************************************************************
## repay

### Description: 
Repays a borrowed debt. Requires a general refresh of all reserves the user
has borrowed or used as collateral

### Arguments:
* Amount - quantity to repay, specified in tokens or loan notes

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Payer - aka UserAddress, typically the user's wallet
* Reserve
* Vault - can be extracted from `reserve`
* Obligation - user's obligation on this market, must be initialized
* LoanNoteMint - can be extracted from `reserve`
* LoanAccount - user's loan account, must be initialized
* PayerAccount - user's payment account, tokens will be transferred from here
 
### Notes:

None

********************************************************************************
## set_market_flags

### Description: 
Adjusts parameters of a Jet Market to halt borrows/withdraws, repays, or withdraws on the entire
market.

The Jet Market owner must sign this tx.

### Arguments:
* Flags - A u64 that encodes market parameters: HALT_BORROWS = 1 << 0, HALT_REPAYS = 1 << 1,
  HALT_DEPOSITS = 1 << 2, supports combinations by ORing or adding together bits. HALT_NOTHING = 0

### Accounts:
* Market - jet market the reserve is under
* Owner - administrator/owner of the jet market, must sign.
 
### Notes:
 
Deposits and Withdraws are disabled/enabled together. 

********************************************************************************
## set_market_owner

### Description: 
Transfers ownership of this Jet Market to a new owner

The Jet Market owner must sign this tx.

### Arguments:
* NewOwner - The public key of the new owner.

### Accounts:
* Market - jet market the reserve is under
* Owner - administrator/owner of the jet market, must sign.
 
### Notes:
 
Passing a bad account as the owner can make this market unusable. Be very careful to specify the
correct new owner.

********************************************************************************
## sync_discount_rates

### Description: 
Tells a reserve to check the discounts account to look for an updated rate.

Permissionless. The market administrator will generally crank this instruction to all reserves as
soon as possible when discount rates are updated, but users may also do so if a reserve is using an
outdated discount rate.

### Arguments:
None

### Accounts:
* Reserve
* Discounts - The discounts account, a program-unique PDA that can be derived from "discounts"
 
### Notes:
 
If the discount rate account has an updated discount rate (a newer version than the reserve
currently uses), updates the rate on the reserve. If the discounts account has no discount rate for the
asset tracked by this reserve, resets it to the default (typically face value of the asset)

********************************************************************************
## update_discounts

### Description: 
Updates the discounts account with a new set of rates.

### Arguments:
MintAddresses - address of the mints that are discounted. An array of `MAX_DISCOUNT_RATES`
PublicKeys, unused slots may be padded with the default PublicKey.

MintAddresses - the discount rate. An array of `MAX_DISCOUNT_RATES` u16s, unused slots may be 
padded with 0. 

### Accounts:
* Authority - TODO the authority that can update the discounts account
* Discounts - The discounts account, a program-unique PDA that can be derived from "discounts"
 
### Notes:
 
If the discount rate account has an updated discount rate (a newer version than the reserve
currently uses), updates the rate on the reserve. If the discounts account has no discount rate for the
asset tracked by this reserve, resets it to the default (typically face value of the asset)

Uses a common denominator of 10,000, e.g. to pass 90%, pass 10,000 * .9 = 9,000. 

A discount rate of 90% means the asset is worth 90% of face value for purposes of computing collateral.

`MAX_DISCOUNT_RATES` is currently 20

********************************************************************************
## update_reserve_config

### Description: 
The market administrator use this ix to update the fees, borrow rates, etc for a reserve.

 Does not update the `deposit_reward_multiplier` and `borrow_reward_multiplier`, use
 `update_reserve_reward` to modify those fields.

The Jet Market owner must sign this tx.

### Arguments:
Config - A reserve config contains the following, which are all numbers unless otherwise specified:

utilizationRate1, utilizationRate2, borrowRate0,
borrowRate1, borrowRate2, borrowRate3, minCollateralRatio, liquidationPremium,
manageFeeCollectionThreshold (a BN), manageFeeRate, loanOriginationFee, liquidationDexTradeMax (a BN), 
confidenceThreshold

All rates use the common denominator of 10,000, e.g.  to pass 1%, pass 10,000  * .01 = 100

### Accounts:
* Market - jet market the reserve is under
* Reserve
* Owner - administrator/owner of the jet market, must sign.
 
### Notes:
 
None.

********************************************************************************
## update_reserve_halts

### Description: 
The market administrator use this ix to halt borrows, repays, deposits, or withdraws on a specific reserve.

The Jet Market owner must sign this tx.

### Arguments:
Halts - A single byte encodes which operations to halt. 0 = resume all, 1 = halt deposits, 2 =
halt borrows, 4 = halt repays, 8 = halt withdraws. Can add to halt multiple, e.g. 1 + 4 = 5, to halt 
deposits and repays only

### Accounts:
* Market - jet market the reserve is under
* Reserve
* Owner - administrator/owner of the jet market, must sign.
 
### Notes:
 
The `set_market_flags` does the same for the entire Jet Market. Unlike the `set_market_flags` ix,
this ix can halt just deposits or just withdraws, without halting both.

********************************************************************************
## update_market_reward

### Description: 
Used for updating a `RewardState` on the `MarketReward` to set the reward token mint and initialize the reward token account for a particular period.

This should be run once for each token to be issued, before the period starts. This can be run in
advance, for any future period, regardless of how far out it is.

### Arguments:
* state_index: period to modify
* info_index: reward info to modify
### Accounts:
* Market - jet market
* Owner - owner of market
* MarketAuthority - PDA to use as authority of token account
* MarketReward - market reward to be initialized
* RewardTokenMint - mint of reward to be distributed
* RewardTokenAccount - token account to be initialized for storing distribution tokens
* SystemProgram
* TokenProgram
* Rent

### Notes:
There cannot be more than one reward token account in a single period sharing the same mint. Reward
tokens have to be transferred to the reward token account after initialization by this method.
Setting a reward period locks in the withdrawal time based on the current `min_withdrawal_duration`
currently stored on the market. A period cannot be updated after the period *starts* (note that the
currently active period also can't be updated), or after it's been initialized already.

********************************************************************************
## update_market_reward_config

### Description: 
Used for updating `MarketRewardState` in Market to update mutable fields (currently only `min_withdrawal_duration`).

### Arguments:
* minWithdrawalDuration - min duration from start of period, after which rewards claim is allowed
### Accounts:
* Market - jet market
* Owner - owner of market

********************************************************************************
## update_reserve_reward

### Description: 
Used for updating `Reserve` to the deposit and borrow reward multipliers that will determine the fractional allocation of reward points to this reserve on supply and borrow side.

### Arguments:
* deposit_reward_multiplier: a number from 0-255
* borrow_reward_multiplier: a number from 0-255
### Accounts:
* Market - jet market
* Owner - owner of market
* Reserve - reserve to modify 

### Notes:
An example of how the multiplier works: If deposit SOL has a multiplier of 5, and total multiplier across all reserve is 50, deposit SOL will be allocated 5/50 of total reward points each period to be allocated pro-rata to SOL depositors.

This ix should be called carefully once reward distribution has started since any reserve that has yet to accrue reward will also be affected when the total multiplier changes. This in turn would cause the total distribution in that period to potentially exceed or fall below the total reward points per period - a behavior that is expected and tolerated by the system.

Our recommendation is to call `accrue_interest` for all reserves first, followed by this instruction, in a single atomic transaction, if possible. 

********************************************************************************
## withdraw_collateral

### Description: 
Moves collateral from the user collateral account to the deposit account.

Only the reserve being withdrawn from needs to be accrued, run the `accrue_interest` ix.

Requires a general refresh of all reserves the user has borrowed or used as collateral

### Arguments:
* Bump - for the user's collateral account
* Amount - quantity to transfer, specified in tokens or deposit notes

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* Depositor - aka UserAddress, typically the user's wallet, must sign tx.
* Obligation - user's obligation on this market, must be initialized
* Reserve
* CollateralAccount - user's collateral account, must be initialized
* DepositAccount - user's deposit account, must be initialized
 
### Notes:
 
Users must withdraw funds to their deposit account using this ix before they can claim the funds back to
their actual token account. After this ix is executed, the funds are considered deposits rather than
collateral, and no longer count as collateral for borrowing purposes. 

The ix will fail if this withdraw of funds would make the obligation unhealthy.

********************************************************************************
## withdraw_tokens, withdraw

### Description: 
Withdraws funds from a deposit account, ultimately giving tokens from the reserve vault back to the 
user's token account. Generally follows `withdraw_collateral`.

Only the reserve being withdrawn from needs to be accrued, run the `accrue_interest` ix.

### Arguments:
* Bump - for the user's deposit account
* Amount - quantity to transfer, specified in tokens or deposit notes

### Accounts:
* Market - jet market the reserve is under
* Market Authority- authority for the jet market
* ReceiverAccount - account where the funds will be sent
* DepositAccount - For withdraw, the user's deposit account, must be initialized. For
  withdraw_tokens, any token account that holds deposit notes.
* Depositor - aka UserAddress, typically the user's wallet, must sign tx.
* Reserve
* Vault - can be extracted from `reserve`, holds the same kind of tokens as the receiver
* DepositNoteMint - can be extracted from `reserve`
 
### Notes:
 
Because these funds are no longer being used as collateral, a general refresh of the user's reserves
is not required. 

These instructions are nearly identical: `withdraw` calls `withdraw_tokens`. The `withdraw_tokens` ix does
not check that the `deposit_account` is a valid PDA of this program, enabling users to redeem deposit
notes from an external token account. A user may wish to do this after they have deposited notes
externally with `deposit_tokens`, generally to use those notes for staking or some other external
purpose. In all other circumstances, use `withdraw`.
