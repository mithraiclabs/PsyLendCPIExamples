// Note: Always run this test with `anchor test -- --features devnet`
// Your wallet must have some SOL and USDC for all tests to pass

import { PsyLend, pdas, types } from "@mithraic-labs/psylend-utils";
import {
  MarketAccount,
  ReserveAccount,
  Obligation,
} from "@mithraic-labs/psylend-utils/dist/types";
import * as anchor from "@project-serum/anchor";
import {
  AnchorProvider,
  Wallet,
  Program,
  workspace,
  BN,
} from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { assert } from "chai";
import { PsylendCpi } from "../target/types/psylend_cpi";
import {
  DEVNET_MAIN_MARKET_KEY,
  MAINNET_MAIN_MARKET_KEY,
  DEVNET_SOL_RESERVE,
  MAINNET_SOL_RESERVE,
  DEVNET_USDC_RESERVE,
  MAINNET_USDC_RESERVE,
} from "./constants";

/**
 * True if using devnet, false if using mainnet/localnet. Some instructions pick which program key
 * to use based on this value.
 */
const isDevnet = true;
const verbose = true;

describe("Generic supply-side yield aggregator example", () => {
  const provider = AnchorProvider.env();
  const con = provider.connection;
  anchor.setProvider(provider);
  const wallet = provider.wallet as Wallet;
  const program: Program<PsylendCpi> = workspace.PsylendCpi;
  const psyLendProgram: Program<PsyLend> = workspace.Psylend;

  let marketKey: PublicKey = isDevnet
    ? new PublicKey(DEVNET_MAIN_MARKET_KEY)
    : new PublicKey(MAINNET_MAIN_MARKET_KEY);
  let market: MarketAccount;
  let marketAuthority: PublicKey;
  let solReserveKey: PublicKey = isDevnet
    ? new PublicKey(DEVNET_SOL_RESERVE)
    : new PublicKey(MAINNET_SOL_RESERVE);
  let usdcReserveKey: PublicKey = isDevnet
    ? new PublicKey(DEVNET_USDC_RESERVE)
    : new PublicKey(MAINNET_USDC_RESERVE);
  let solReserve: ReserveAccount;
  let usdcReserve: ReserveAccount;

  let obligationKey: PublicKey;
  let obligationBump: number;
  let obligation: Obligation;

  /**
   * The ATA that holds USDC for this wallet
   */
  let usdcTokenAccountKey: PublicKey;
  /**
   * Deposit note account for USDC. Not PsyLend owned, for cases where an integrator wants custody
   * of their own deposit notes, and the ability to move them around at will.
   *
   * Notes in this account are not able to be used as collateral.
   */
  let usdcDepositAccountKey: PublicKey;

  /**
   * Many instructions, for example deposit, require reserves to be accrued beforehand.
   */
  let accrueUsdcIx: TransactionInstruction;
  let refreshUsdcIx: TransactionInstruction;

  before(async () => {
    let fetchMarket = psyLendProgram.account.market.fetch(marketKey);
    let fetchSolReserve = psyLendProgram.account.reserve.fetch(solReserveKey);
    let fetchUsdcReserve = psyLendProgram.account.reserve.fetch(usdcReserveKey);
    // ...fetch other Psylend stuff asyncly

    // TODO replace with sync func from PDAs from package after bump
    let _bump: number;
    [marketAuthority, _bump] = await pdas.deriveMarketAuthority(
      // @ts-ignore
      psyLendProgram,
      marketKey
    );

    market = await fetchMarket;
    solReserve = await fetchSolReserve;
    usdcReserve = await fetchUsdcReserve;

    // Various instructions need to accrue interest before they can be called. Send this ix in the
    // same tx.
    accrueUsdcIx = await program.methods
      .accrueInterestCpi()
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        feeNoteVault: usdcReserve.feeNoteVault,
        depositNoteMint: usdcReserve.depositNoteMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();
    refreshUsdcIx = await program.methods
      .refreshReserveCpi()
      .accounts({
        market: marketKey,
        reserve: usdcReserveKey,
        pythOraclePrice: usdcReserve.pythOraclePrice,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    // The user's usdc token account
    usdcTokenAccountKey = getAssociatedTokenAddressSync(
      usdcReserve.tokenMint,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    let usdcAcc = await getAccount(con, usdcTokenAccountKey);

    if (verbose) {
      const bal = await con.getBalance(wallet.publicKey);
      console.log("Cluster: " + con.rpcEndpoint);
      console.log("Program id: " + program.programId);
      console.log("Psylend id: " + psyLendProgram.programId);
      console.log("");
      console.log("Wallet key: " + wallet.publicKey);
      console.log("wallet initial SOL balance: " + bal.toLocaleString());
      console.log(
        "wallet initial USDC balance: " + usdcAcc.amount.toLocaleString()
      );
      console.log("");
      console.log("Market key: " + marketKey);
      console.log("Market auth: " + marketAuthority);
      console.log("");
    }
  });

  it("Read the current interest rate on some reserve", async () => {
    const ix = await program.methods
      .getCurrentInterest()
      .accounts({
        market: marketKey,
        reserve: usdcReserveKey,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }
    // Check the program log to see the message.
  });

  it("Creates external USDC deposit account (not owned by PsyLend)", async () => {
    // Deposit notes will be go into this account after a deposit
    usdcDepositAccountKey = getAssociatedTokenAddressSync(
      usdcReserve.depositNoteMint,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    try {
      await provider.sendAndConfirm(
        new Transaction().add(
          createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            usdcDepositAccountKey,
            wallet.publicKey,
            usdcReserve.depositNoteMint,
            TOKEN_PROGRAM_ID,
            ASSOCIATED_TOKEN_PROGRAM_ID
          )
        )
      );
    } catch (err) {
      // this test doesn't close the deposit note account, so after the first run, this tx
      // will fail as the acc already exists.
    }

    // Exists
    try {
      await getAccount(con, usdcDepositAccountKey);
      assert.ok(true);
    } catch (err) {
      assert.ok(false);
    }
  });

  it("Deposits 1 USDC into PsyLend, notes go to external acc", async () => {
    let usdcAccountBefore = await getAccount(con, usdcTokenAccountKey);
    let depositAccountBefore = await getAccount(con, usdcDepositAccountKey);

    let amount = types.Amount.tokens(
      new BN(1 * 10 ** Math.abs(usdcReserve.exponent))
    );

    const ix = await program.methods
      .depositTokensCpi(amount)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        depositSource: usdcTokenAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(accrueUsdcIx, ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    let usdcAccountAfter = await getAccount(con, usdcTokenAccountKey);
    let depositAccountAfter = await getAccount(con, usdcDepositAccountKey);

    if (verbose) {
      console.log(
        "USDC before: " +
          usdcAccountBefore.amount.toLocaleString() +
          " after: " +
          usdcAccountAfter.amount.toLocaleString()
      );
      console.log(
        "USDC notes: " +
          depositAccountBefore.amount.toLocaleString() +
          " after: " +
          depositAccountAfter.amount.toLocaleString()
      );
    }

    assert.isAbove(
      Number(depositAccountAfter.amount),
      Number(depositAccountBefore.amount)
    );
    assert.isAbove(
      Number(usdcAccountBefore.amount),
      Number(usdcAccountAfter.amount)
    );
  });

  it("Withdraws all USDC from PsyLend, with notes from external acc", async () => {
    let usdcAccountBefore = await getAccount(con, usdcTokenAccountKey);
    let depositAccountBefore = await getAccount(con, usdcDepositAccountKey);

    // Due to interest, the notes will be worth slightly more usdc than the initial deposit.
    let amount = types.Amount.depositNotes(
      new BN(depositAccountBefore.amount.toString())
    );

    const ix = await program.methods
      .withdrawTokensCpi(amount)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        withdrawAccount: usdcTokenAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(accrueUsdcIx, ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    let usdcAccountAfter = await getAccount(con, usdcTokenAccountKey);
    let depositAccountAfter = await getAccount(con, usdcDepositAccountKey);

    if (verbose) {
      console.log(
        "USDC before: " +
          usdcAccountBefore.amount.toLocaleString() +
          " after: " +
          usdcAccountAfter.amount.toLocaleString()
      );
      console.log(
        "USDC notes: " +
          depositAccountBefore.amount.toLocaleString() +
          " after: " +
          depositAccountAfter.amount.toLocaleString()
      );
    }

    assert.isBelow(
      Number(depositAccountAfter.amount),
      Number(depositAccountBefore.amount)
    );
    assert.isBelow(
      Number(usdcAccountBefore.amount),
      Number(usdcAccountAfter.amount)
    );
  });

  it("Combination instruction examples: deposit/withdraw USDC in one CPI ix", async () => {
    let amount = types.Amount.tokens(
      new BN(1 * 10 ** Math.abs(usdcReserve.exponent))
    );

    const depositIx = await program.methods
      .accrueDepositTokensCpi(amount)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        depositSource: usdcTokenAccountKey,
        feeNoteVault: usdcReserve.feeNoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      // Note the lack of an accrue ix added to the tx here.
      await provider.sendAndConfirm(new Transaction().add(depositIx));
    } catch (err) {
      console.log(err);
      throw err;
    }

    // Even on this short timescale, interest can cause the amount to differ.
    let depositAccount = await getAccount(con, usdcDepositAccountKey);

    amount = types.Amount.depositNotes(
      new BN(depositAccount.amount.toString())
    );

    const withdrawIx = await program.methods
      .accrueWithdrawTokensCpi(amount)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        withdrawAccount: usdcTokenAccountKey,
        feeNoteVault: usdcReserve.feeNoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(withdrawIx));
    } catch (err) {
      console.log(err);
      throw err;
    }
  });
});
