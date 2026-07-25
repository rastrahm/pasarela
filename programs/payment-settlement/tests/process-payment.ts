/**
 * TDD — casos borde obligatorios (Arquitectura §7.4).
 *
 * Estado esperado tras 3.2: tests compilados; casos de `process_payment` en ROJO
 * hasta Fase 3.4 (lógica) y 3.6 (constraints).
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { expect } from "chai";
import { PaymentSettlement } from "../target/types/payment_settlement";
import { expectAnchorError, expectTxSuccess } from "./helpers/errors";
import {
  BRAND_VISA,
  RAIL_SOLANA_WALLET,
  processPaymentAccounts,
  processPaymentSigners,
  readTokenBalance,
  setupProcessPaymentContext,
} from "./helpers/token";

describe("process_payment (TDD §7.4)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.paymentSettlement as Program<PaymentSettlement>;
  const provider = anchor.AnchorProvider.env();

  /** Crea una cuenta demasiado pequeña para alojar `SettlementState`. */
  async function createUndersizedSettlementAccount(
    payer: Keypair,
  ): Promise<PublicKey> {
    const undersized = Keypair.generate();
    const space = 16; // menor que SettlementState::LEN (57)
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(
      space,
    );

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: undersized.publicKey,
        lamports,
        space,
        programId: program.programId,
      }),
    );

    await sendAndConfirmTransaction(provider.connection, tx, [payer, undersized]);
    return undersized.publicKey;
  }

  it("rejects payment signed by an unauthorized user", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const attacker = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      attacker.publicKey,
      anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    const amount = new anchor.BN(100_000);

    try {
      await program.methods
        .processPayment(amount, BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(
          processPaymentAccounts(ctx, { payer: attacker.publicKey }),
        )
        .signers([attacker])
        .rpc();
      expect.fail("expected Unauthorized");
    } catch (err) {
      expectAnchorError(err, "Unauthorized");
    }
  });

  it("rejects payment when settlement account has insufficient space", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const undersizedSettlement = await createUndersizedSettlementAccount(
      ctx.tokens.payer,
    );

    const amount = new anchor.BN(50_000);

    try {
      await program.methods
        .processPayment(amount, BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(
          processPaymentAccounts(ctx, {
            settlementState: undersizedSettlement,
          }),
        )
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected InsufficientAccountSpace");
    } catch (err) {
      expectAnchorError(err, "InsufficientAccountSpace");
    }
  });

  it("rejects payment when amount would overflow settlement counters", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);

    // Pre-cargar contador cerca de u64::MAX para forzar overflow en total_amount.
    const nearMax = new anchor.BN("18446744073709551610"); // u64::MAX - 5
    await program.methods
      .setSettlementTotals(nearMax, new anchor.BN(1))
      .accountsPartial({
        settlementState: ctx.settlementState,
        merchant: ctx.tokens.merchant.publicKey,
      })
      .signers([ctx.tokens.merchant])
      .rpc();

    const overflowAmount = new anchor.BN(10);

    try {
      await program.methods
        .processPayment(overflowAmount, BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(processPaymentAccounts(ctx))
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected AmountOverflow");
    } catch (err) {
      expectAnchorError(err, "AmountOverflow");
    }
  });

  it("transfers SPL, updates settlement counters and emits PaymentProcessed", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const amount = new anchor.BN(250_000);
    const payerBefore = await readTokenBalance(
      provider.connection,
      ctx.tokens.payerAta,
    );
    const merchantBefore = await readTokenBalance(
      provider.connection,
      ctx.tokens.merchantAta,
    );

    const events: anchor.IdlEvents<PaymentSettlement>["paymentProcessed"][] = [];
    const listener = program.addEventListener(
      "paymentProcessed",
      (event, _slot) => {
        events.push(event);
      },
    );

    try {
      const tx = await program.methods
        .processPayment(amount, BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(processPaymentAccounts(ctx))
        .signers(processPaymentSigners(ctx))
        .rpc();
      expectTxSuccess(tx);

      const payerAfter = await readTokenBalance(
        provider.connection,
        ctx.tokens.payerAta,
      );
      const merchantAfter = await readTokenBalance(
        provider.connection,
        ctx.tokens.merchantAta,
      );

      expect(payerBefore - payerAfter).to.equal(BigInt(amount.toString()));
      expect(merchantAfter - merchantBefore).to.equal(BigInt(amount.toString()));

      const state = await program.account.settlementState.fetch(
        ctx.settlementState,
      );
      expect(state.totalAmount.toString()).to.equal(amount.toString());
      expect(state.paymentCount.toNumber()).to.equal(1);

      expect(events).to.have.lengthOf(1);
      expect(events[0].amount.toString()).to.equal(amount.toString());
      expect(events[0].brandCode).to.equal(BRAND_VISA);
      expect(events[0].settlementRailId.toString()).to.equal(
        RAIL_SOLANA_WALLET.toString(),
      );
      expect(events[0].timestamp.toNumber()).to.be.greaterThan(0);
    } finally {
      await program.removeEventListener(listener);
    }
  });
});
