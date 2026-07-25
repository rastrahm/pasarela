import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { PaymentSettlement } from "../../target/types/payment_settlement";

export const BRAND_VISA = 1;
export const RAIL_SOLANA_WALLET = new anchor.BN(3);

export interface TokenFixture {
  mint: PublicKey;
  payer: Keypair;
  merchant: Keypair;
  payerAta: PublicKey;
  merchantAta: PublicKey;
}

export interface ProcessPaymentContext {
  program: Program<PaymentSettlement>;
  provider: anchor.AnchorProvider;
  tokens: TokenFixture;
  settlementState: PublicKey;
  settlementBump: number;
}

export async function createTokenFixture(
  provider: anchor.AnchorProvider,
  payerBalance: bigint,
): Promise<TokenFixture> {
  const payer = Keypair.generate();
  const merchant = Keypair.generate();

  const airdropLamports = async (keypair: Keypair) => {
    const sig = await provider.connection.requestAirdrop(
      keypair.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);
  };

  await airdropLamports(payer);
  await airdropLamports(merchant);

  const mint = await createMint(
    provider.connection,
    payer,
    payer.publicKey,
    null,
    6,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID,
  );

  const payerAta = await createAssociatedTokenAccount(
    provider.connection,
    payer,
    mint,
    payer.publicKey,
    undefined,
    TOKEN_PROGRAM_ID,
  );

  const merchantAta = await createAssociatedTokenAccount(
    provider.connection,
    payer,
    mint,
    merchant.publicKey,
    undefined,
    TOKEN_PROGRAM_ID,
  );

  if (payerBalance > 0n) {
    await mintTo(
      provider.connection,
      payer,
      mint,
      payerAta,
      payer,
      payerBalance,
      undefined,
      TOKEN_PROGRAM_ID,
    );
  }

  return { mint, payer, merchant, payerAta, merchantAta };
}

export function settlementPda(
  programId: PublicKey,
  merchant: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("settlement"), merchant.toBuffer()],
    programId,
  );
}

export async function initSettlementState(
  program: Program<PaymentSettlement>,
  merchant: PublicKey,
  payer: Keypair,
): Promise<{ settlementState: PublicKey; bump: number }> {
  const [settlementState, bump] = settlementPda(program.programId, merchant);

  await program.methods
    .initializeSettlement(bump)
    .accountsPartial({
      merchant,
      settlementState,
      payer: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();

  return { settlementState, bump };
}

export async function readTokenBalance(
  connection: anchor.web3.Connection,
  ata: PublicKey,
): Promise<bigint> {
  const account = await getAccount(connection, ata, undefined, TOKEN_PROGRAM_ID);
  return account.amount;
}

export async function setupProcessPaymentContext(
  program: Program<PaymentSettlement>,
  provider: anchor.AnchorProvider,
  payerBalance: bigint = 1_000_000n,
): Promise<ProcessPaymentContext> {
  const tokens = await createTokenFixture(provider, payerBalance);
  const { settlementState, bump } = await initSettlementState(
    program,
    tokens.merchant.publicKey,
    tokens.payer,
  );

  return {
    program,
    provider,
    tokens,
    settlementState,
    settlementBump: bump,
  };
}

export function processPaymentAccounts(
  ctx: ProcessPaymentContext,
  overrides: {
    payer?: PublicKey;
    payerTokenAccount?: PublicKey;
    merchantTokenAccount?: PublicKey;
    merchant?: PublicKey;
    settlementState?: PublicKey;
  } = {},
) {
  return {
    payer: overrides.payer ?? ctx.tokens.payer.publicKey,
    payerTokenAccount: overrides.payerTokenAccount ?? ctx.tokens.payerAta,
    merchantTokenAccount:
      overrides.merchantTokenAccount ?? ctx.tokens.merchantAta,
    merchant: overrides.merchant ?? ctx.tokens.merchant.publicKey,
    settlementState: overrides.settlementState ?? ctx.settlementState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  };
}

export function processPaymentSigners(
  ctx: ProcessPaymentContext,
  extra: Keypair[] = [],
): Keypair[] {
  return [ctx.tokens.payer, ...extra];
}
