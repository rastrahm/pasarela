/**
 * Fase 3.6 — constraints explícitos en #[derive(Accounts)].
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { PaymentSettlement } from "../target/types/payment_settlement";
import { expectAnchorError } from "./helpers/errors";
import {
  BRAND_VISA,
  RAIL_SOLANA_WALLET,
  initSettlementState,
  processPaymentAccounts,
  processPaymentSigners,
  setupProcessPaymentContext,
  settlementPda,
} from "./helpers/token";

describe("process_payment constraints (Fase 3.6)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.paymentSettlement as Program<PaymentSettlement>;
  const provider = anchor.AnchorProvider.env();

  it("rejects payment when payer and merchant token mints differ", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const otherMint = await createMint(
      provider.connection,
      ctx.tokens.payer,
      ctx.tokens.payer.publicKey,
      null,
      6,
    );
    const wrongMintAta = await createAssociatedTokenAccount(
      provider.connection,
      ctx.tokens.payer,
      otherMint,
      ctx.tokens.payer.publicKey,
    );
    await mintTo(
      provider.connection,
      ctx.tokens.payer,
      otherMint,
      wrongMintAta,
      ctx.tokens.payer,
      500_000,
    );

    try {
      await program.methods
        .processPayment(new anchor.BN(100_000), BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(
          processPaymentAccounts(ctx, { payerTokenAccount: wrongMintAta }),
        )
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected InvalidMint");
    } catch (err) {
      expectAnchorError(err, "InvalidMint");
    }
  });

  it("rejects payment when amount is zero", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);

    try {
      await program.methods
        .processPayment(new anchor.BN(0), BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(processPaymentAccounts(ctx))
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected InvalidAmount");
    } catch (err) {
      expectAnchorError(err, "InvalidAmount");
    }
  });

  it("rejects payment when payer token balance is insufficient", async () => {
    const ctx = await setupProcessPaymentContext(program, provider, 50_000n);

    try {
      await program.methods
        .processPayment(new anchor.BN(100_000), BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(processPaymentAccounts(ctx))
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected InsufficientFunds");
    } catch (err) {
      expectAnchorError(err, "InsufficientFunds");
    }
  });

  it("rejects payment when settlement PDA belongs to another merchant", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const otherMerchant = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      otherMerchant.publicKey,
      anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    const { settlementState: otherSettlement } = await initSettlementState(
      program,
      otherMerchant.publicKey,
      ctx.tokens.payer,
    );

    try {
      await program.methods
        .processPayment(new anchor.BN(10_000), BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(
          processPaymentAccounts(ctx, { settlementState: otherSettlement }),
        )
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected Unauthorized or ConstraintSeeds");
    } catch (err) {
      const code = (err as anchor.AnchorError).error?.errorCode?.code;
      expect(code).to.be.oneOf(["Unauthorized", "ConstraintSeeds"]);
    }
  });

  it("rejects payment when settlement address does not match merchant seeds", async () => {
    const ctx = await setupProcessPaymentContext(program, provider);
    const impostorMerchant = Keypair.generate().publicKey;
    const [wrongPda] = settlementPda(program.programId, impostorMerchant);

    try {
      await program.methods
        .processPayment(new anchor.BN(10_000), BRAND_VISA, RAIL_SOLANA_WALLET)
        .accounts(
          processPaymentAccounts(ctx, {
            merchant: ctx.tokens.merchant.publicKey,
            settlementState: wrongPda,
          }),
        )
        .signers(processPaymentSigners(ctx))
        .rpc();
      expect.fail("expected ConstraintSeeds or InvalidSettlementPda");
    } catch (err) {
      const code = (err as anchor.AnchorError).error?.errorCode?.code;
      expect(code).to.be.oneOf(["ConstraintSeeds", "InvalidSettlementPda"]);
    }
  });
});
