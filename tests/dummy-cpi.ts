// A bare minimum CPI example using a trivial program.
import * as anchor from "@project-serum/anchor";
import {
  AnchorProvider,
  Wallet,
  Program,
  workspace,
} from "@project-serum/anchor";
import { PsylendCpi } from "../target/types/psylend_cpi";
import { CpiDummy } from "../deps/cpi_dummy";
import { Transaction } from "@solana/web3.js";
import { assert } from "chai";

describe("Bare Minimum CPI example", () => {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet as Wallet;
  const program: Program<PsylendCpi> = workspace.PsylendCpi;
  const dummyProgram: Program<CpiDummy> = workspace.CpiDummy;

  it("Calls a basic CPI", async () => {
    const ix = await program.methods
      .dummyCpi()
      .accounts({
        dummyAcc: wallet.publicKey,
        dummyProgram: dummyProgram.programId,
      })
      .instruction();

    let tx: Transaction = new Transaction();
    tx.add(ix);
    try {
      await provider.sendAndConfirm(tx);
      assert.ok(true);
    } catch (err) {
      console.log(err);
      assert.ok(false);
      throw err;
    }
    console.log("done!");
  });
});
