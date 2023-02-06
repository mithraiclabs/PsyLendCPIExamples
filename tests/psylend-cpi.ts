import { pdas, PsyLend, types } from "@mithraic-labs/psylend-utils";
import { deriveMarketAuthority } from "@mithraic-labs/psylend-utils/dist/pdas";
import {
  MarketAccount,
  Obligation,
  ReserveAccount,
} from "@mithraic-labs/psylend-utils/dist/types";
import * as anchor from "@project-serum/anchor";
import {
  AnchorProvider,
  BN,
  Program,
  Wallet,
  workspace,
} from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  TYPE_SIZE,
} from "@solana/spl-token";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { assert } from "chai";
import { PsylendCpi } from "../target/types/psylend_cpi";
import {
  DEVNET_BTC_PUT_RESERVE,
  DEVNET_MAIN_MARKET_KEY,
  DEVNET_SOL_RESERVE,
  DEVNET_USDC_RESERVE,
  MAINNET_BTC_PUT_RESERVE,
  MAINNET_MAIN_MARKET_KEY,
  MAINNET_SOL_RESERVE,
  MAINNET_USDC_RESERVE,
} from "./constants";

/**
 * True if using devnet, false if using mainnet/localnet. Some instructions pick which program key
 * to use based on this value.
 */
const isDevnet = true;
const verbose = true;

describe("PsyLend CPI examples", () => {
  const provider = AnchorProvider.env();
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
  let btcPutReserveKey: PublicKey = isDevnet
    ? new PublicKey(DEVNET_BTC_PUT_RESERVE)
    : new PublicKey(MAINNET_BTC_PUT_RESERVE);
  let solReserve: ReserveAccount;
  let usdcReserve: ReserveAccount;
  let btcPutReserve: ReserveAccount;

  let obligationKey: PublicKey;
  let obligationBump: number;
  let obligation: Obligation;

  /**
   * The ATA that holds USDC for this wallet
   */
  let usdcTokenAccountKey: PublicKey;
  /**
   * Deposit note account for USDC, a pda opened with initDepositAccount.
   * Notes in this account are not able to be used as collateral
   */
  let usdcDepositAccountKey: PublicKey;
  let usdcDepositAccountBump: number;
  /**
   * Collateral note account for USDC, a pda opened with initCollateralAccount.
   * Notes in this account can be used as collateral to borrow
   */
  let usdcCollateralAccountKey: PublicKey;
  let usdcCollateralAccountBump: number;

  /**
   * Many instructions, for example deposit, require reserves to be accrued beforehand.
   */
  let accrueUsdcIx: TransactionInstruction;

  before(async () => {
    let fetchMarket = psyLendProgram.account.market.fetch(marketKey);
    let fetchSolReserve = psyLendProgram.account.reserve.fetch(solReserveKey);
    let fetchUsdcReserve = psyLendProgram.account.reserve.fetch(usdcReserveKey);
    let fetchBtcPutReserve =
      psyLendProgram.account.reserve.fetch(btcPutReserveKey);
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
    btcPutReserve = await fetchBtcPutReserve;

    if (verbose) {
      const bal = await provider.connection.getBalance(wallet.publicKey);
      console.log("Cluster: " + provider.connection.rpcEndpoint);
      console.log("Program id: " + program.programId);
      console.log("Psylend id: " + psyLendProgram.programId);
      console.log("");
      console.log("Wallet key: " + wallet.publicKey);
      console.log("wallet initial SOL balance: " + bal);
      console.log("");
      console.log("Market key: " + marketKey);
      console.log("Market auth: " + marketAuthority);
      console.log("");
    }
  });

  it("Refreshes a reserve by CPI", async () => {
    const ix = await program.methods
      .refreshReserveCpi()
      .accounts({
        market: marketKey,
        reserve: solReserveKey,
        pythOraclePrice: solReserve.pythOraclePrice,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }
  });

  it("Refreshes a PsyFi reserve by CPI", async () => {
    const ix = await program.methods
      .refreshPsyfiReserveCpi()
      .accounts({
        market: marketKey,
        reserve: btcPutReserveKey,
        psyfiVaultAccount: btcPutReserve.psyfiVaultConfig.vaultAccount,
        pythOraclePrice: btcPutReserve.pythOraclePrice,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }
  });

  it("Inits obligation by CPI", async () => {
    // Derive the obligation account first, e.g:
    ({ obligationAccount: obligationKey, obligationBump } =
      await pdas.deriveUserObligation(
        // @ts-ignore
        psyLendProgram,
        marketKey,
        wallet.publicKey
      ));

    const ix = await program.methods
      .initObligationCpi(obligationBump)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        obligation: obligationKey,
        borrower: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    obligation = await psyLendProgram.account.obligation.fetch(obligationKey);
    assert.equal(obligation.owner.toString(), wallet.publicKey.toString());
    if (verbose) {
      console.log("created an obligation: " + obligationKey);
    }
  });

  it("Inits deposit account (USDC) by CPI", async () => {
    // Derive the account address before creating it, e.g.
    // TODO replace with call from /pdas after package bump
    [usdcDepositAccountKey, usdcDepositAccountBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("deposits"),
          usdcReserveKey.toBytes(),
          wallet.publicKey.toBytes(),
        ],
        psyLendProgram.programId
      );
    if (verbose) {
      console.log("creating usdc deposit acc: " + usdcDepositAccountKey);
    }

    const ix = await program.methods
      .initDepositCpi(usdcDepositAccountBump)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }
  });

  it("Executes a deposit (1 USDC) by CPI", async () => {
    // Note: Deposit source is generally an ATA, derive for the users's wallet, e.g:
    usdcTokenAccountKey = getAssociatedTokenAddressSync(
      usdcReserve.tokenMint,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    let usdcAccountBefore = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );
    let depositAccountBefore = await getAccount(
      provider.connection,
      usdcDepositAccountKey
    );

    let amount = types.Amount.tokens(
      new BN(1 * 10 ** Math.abs(usdcReserve.exponent))
    );
    accrueUsdcIx = await program.methods
      .acrrueInterestCpi()
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

    const depositIx = await program.methods
      .depositCpi(usdcDepositAccountBump, amount)
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
      await provider.sendAndConfirm(
        new Transaction().add(accrueUsdcIx, depositIx)
      );
    } catch (err) {
      console.log(err);
      throw err;
    }

    let usdcAccountAfter = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );
    let depositAccountAfter = await getAccount(
      provider.connection,
      usdcDepositAccountKey
    );

    if (verbose) {
      console.log(
        "usdc before deposit: " +
          usdcAccountBefore.amount.toLocaleString() +
          " after " +
          usdcAccountAfter.amount.toLocaleString()
      );
      console.log(
        "deposit notes before deposit: " +
          depositAccountBefore.amount.toLocaleString() +
          " after " +
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

  it("Inits collateral account (USDC) by CPI", async () => {
    // Derive the account address before creating it, e.g.
    // TODO replace with call from /pdas after package bump
    [usdcCollateralAccountKey, usdcCollateralAccountBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("collateral"),
          usdcReserveKey.toBytes(),
          obligationKey.toBytes(),
          wallet.publicKey.toBytes(),
        ],
        psyLendProgram.programId
      );
    if (verbose) {
      console.log("creating usdc collateral acc: " + usdcCollateralAccountKey);
    }

    const ix = await program.methods
      .initCollateralAccountCpi(usdcCollateralAccountBump)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        obligation: obligationKey,
        reserve: usdcReserveKey,
        depositNoteMint: usdcReserve.depositNoteMint,
        owner: wallet.publicKey,
        collateralAccount: usdcCollateralAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    // Exists
    try {
      await getAccount(provider.connection, usdcDepositAccountKey);
      assert.ok(true);
    } catch (err) {
      assert.ok(false);
    }
  });

  it("Closes collateral account (USDC) by CPI", async () => {
    // Note that the collateral account must be empty.
    const ix = await program.methods
      .closeCollateralAccountCpi()
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        obligation: obligationKey,
        owner: wallet.publicKey,
        collateralAccount: usdcCollateralAccountKey,
        depositAccount: usdcDepositAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    // Doesn't exist
    try {
      await getAccount(provider.connection, usdcDepositAccountKey);
      assert.ok(false);
    } catch (err) {
      if (verbose) {
        console.log(usdcDepositAccountKey + " doesn't exist");
      }
      assert.ok(true);
    }
  });

  it("Executes a withdraw (.5 USDC) by CPI", async () => {
    let usdcAccountBefore = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );
    let depositAccountBefore = await getAccount(
      provider.connection,
      usdcDepositAccountKey
    );

    let amount = types.Amount.tokens(
      new BN(0.5 * 10 ** Math.abs(usdcReserve.exponent))
    );
    const ix = await program.methods
      .withdrawCpi(usdcDepositAccountBump, amount)
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        withdrawAccount: usdcTokenAccountKey,
        psyProgram: psyLendProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    let usdcAccountAfter = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );
    let depositAccountAfter = await getAccount(
      provider.connection,
      usdcDepositAccountKey
    );

    if (verbose) {
      console.log(
        "usdc before withdraw: " +
          usdcAccountBefore.amount.toLocaleString() +
          " after " +
          usdcAccountAfter.amount.toLocaleString()
      );
      console.log(
        "deposit notes before withdraw: " +
          depositAccountBefore.amount.toLocaleString() +
          " after " +
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

  it("Closes deposit account (.5 USDC remaining) by CPI", async () => {
    let usdcAccountBefore = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );

    // exists
    try {
      await getAccount(provider.connection, usdcDepositAccountKey);
      assert.ok(true);
    } catch (err) {
      assert.ok(false);
    }

    const ix = await program.methods
      .closeDepositCpi()
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        reserve: usdcReserveKey,
        vault: usdcReserve.vault,
        depositNoteMint: usdcReserve.depositNoteMint,
        depositor: wallet.publicKey,
        depositAccount: usdcDepositAccountKey,
        receiverAccount: usdcTokenAccountKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    let usdcAccountAfter = await getAccount(
      provider.connection,
      usdcTokenAccountKey
    );
    // Doesn't exist
    try {
      await getAccount(provider.connection, usdcDepositAccountKey);
      assert.ok(false);
    } catch (err) {
      assert.ok(true);
    }

    if (verbose) {
      console.log(
        "usdc before close deposit acc: " +
          usdcAccountBefore.amount.toLocaleString() +
          " after " +
          usdcAccountAfter.amount.toLocaleString()
      );
    }

    assert.isBelow(
      Number(usdcAccountBefore.amount),
      Number(usdcAccountAfter.amount)
    );
  });

  it("Closes obligation by CPI", async () => {
    const ix = await program.methods
      .closeObligationCpi()
      .accounts({
        market: marketKey,
        marketAuthority: marketAuthority,
        owner: wallet.publicKey,
        obligation: obligationKey,
        psylendProgram: psyLendProgram.programId,
      })
      .instruction();

    try {
      await provider.sendAndConfirm(new Transaction().add(ix));
    } catch (err) {
      console.log(err);
      throw err;
    }

    try {
      obligation = await psyLendProgram.account.obligation.fetch(obligationKey);
      assert.ok(false);
    } catch (err) {
      // Fails, acc doesn't exist
      assert.ok(true);
    }

    if (verbose) {
      console.log(
        "closed an obligation: " +
          obligationKey +
          " and returned rent to " +
          wallet.publicKey
      );
    }
  });

  it("End of test information", async () => {
    if (verbose) {
      const bal = await provider.connection.getBalance(wallet.publicKey);
      console.log("Test suite done.");
      console.log("wallet final SOL balance: " + bal);
    }
  });
});
