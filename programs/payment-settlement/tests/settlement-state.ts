/**
 * Fase 3.3 — PDA `SettlementState`.
 * Seeds: ["settlement", merchant.key()]
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { PaymentSettlement } from "../target/types/payment_settlement";
import { initSettlementState, settlementPda } from "./helpers/token";

/** Tamaño on-chain esperado: 8 (discriminator) + InitSpace fields. */
export const SETTLEMENT_STATE_LEN = 57;

describe("SettlementState PDA (Fase 3.3)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.paymentSettlement as Program<PaymentSettlement>;
  const provider = anchor.AnchorProvider.env();

  async function fund(keypair: Keypair): Promise<void> {
    const sig = await provider.connection.requestAirdrop(
      keypair.publicKey,
      anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);
  }

  it("derives deterministic address from seeds [settlement, merchant]", () => {
    const merchant = Keypair.generate().publicKey;
    const [pdaA, bumpA] = settlementPda(program.programId, merchant);
    const [pdaB, bumpB] = settlementPda(program.programId, merchant);

    expect(pdaA.toBase58()).to.equal(pdaB.toBase58());
    expect(bumpA).to.equal(bumpB);
    expect(bumpA).to.be.at.most(255);
  });

  it("assigns distinct PDAs to different merchants", () => {
    const merchantA = Keypair.generate().publicKey;
    const merchantB = Keypair.generate().publicKey;
    const [pdaA] = settlementPda(program.programId, merchantA);
    const [pdaB] = settlementPda(program.programId, merchantB);

    expect(pdaA.toBase58()).to.not.equal(pdaB.toBase58());
  });

  it("initialize_settlement creates account with zero counters at expected PDA", async () => {
    const payer = Keypair.generate();
    const merchant = Keypair.generate();
    await fund(payer);

    const [expectedPda, expectedBump] = settlementPda(
      program.programId,
      merchant.publicKey,
    );

    await program.methods
      .initializeSettlement()
      .accountsPartial({
        payer: payer.publicKey,
        merchant: merchant.publicKey,
        settlementState: expectedPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    const state = await program.account.settlementState.fetch(expectedPda);
    expect(state.merchant.toBase58()).to.equal(merchant.publicKey.toBase58());
    expect(state.totalAmount.toNumber()).to.equal(0);
    expect(state.paymentCount.toNumber()).to.equal(0);
    expect(state.bump).to.equal(expectedBump);

    const info = await provider.connection.getAccountInfo(expectedPda);
    expect(info).to.not.be.null;
    expect(info!.owner.toBase58()).to.equal(program.programId.toBase58());
    expect(info!.data.length).to.equal(SETTLEMENT_STATE_LEN);
  });

  it("rejects second initialize for the same merchant", async () => {
    const payer = Keypair.generate();
    const merchant = Keypair.generate();
    await fund(payer);

    const { settlementState } = await initSettlementState(
      program,
      merchant.publicKey,
      payer,
    );

    try {
      await program.methods
        .initializeSettlement()
        .accountsPartial({
          payer: payer.publicKey,
          merchant: merchant.publicKey,
          settlementState,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc();
      expect.fail("expected duplicate init to fail");
    } catch (err) {
      const anchorErr = err as { message?: string };
      expect(anchorErr.message ?? String(err)).to.match(
        /already in use|AccountNotInitialized|custom program error/i,
      );
    }
  });

  it("rejects initialize when settlement_state address does not match seeds", async () => {
    const payer = Keypair.generate();
    const merchant = Keypair.generate();
    const wrongMerchant = Keypair.generate();
    await fund(payer);

    const [wrongPda] = settlementPda(program.programId, wrongMerchant.publicKey);

    try {
      await program.methods
        .initializeSettlement()
        .accountsPartial({
          payer: payer.publicKey,
          merchant: merchant.publicKey,
          settlementState: wrongPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc();
      expect.fail("expected ConstraintSeeds or InvalidSettlementPda");
    } catch (err) {
      const msg = (err as { message?: string }).message ?? String(err);
      expect(msg).to.match(/ConstraintSeeds|InvalidSettlementPda|Seeds do not match/i);
    }
  });

  it("set_settlement_totals requires merchant signer matching PDA", async () => {
    const payer = Keypair.generate();
    const merchant = Keypair.generate();
    const impostor = Keypair.generate();
    await fund(payer);

    const { settlementState } = await initSettlementState(
      program,
      merchant.publicKey,
      payer,
    );

    try {
      await program.methods
        .setSettlementTotals(new anchor.BN(100), new anchor.BN(1))
        .accountsPartial({
          merchant: impostor.publicKey,
          settlementState,
        })
        .signers([impostor])
        .rpc();
      expect.fail("expected Unauthorized or ConstraintSeeds");
    } catch (err) {
      const msg = (err as { message?: string }).message ?? String(err);
      expect(msg).to.match(/Unauthorized|ConstraintSeeds|ConstraintHasOne|2006|2012/i);
    }
  });
});
