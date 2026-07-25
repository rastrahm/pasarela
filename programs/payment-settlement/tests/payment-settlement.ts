//! Bootstrap del workspace Anchor — verifica compilación e IDL (Fase 3.1).

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PaymentSettlement } from "../target/types/payment_settlement";
import { expect } from "chai";

describe("payment-settlement", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.paymentSettlement as Program<PaymentSettlement>;

  it("program id matches Anchor.toml", () => {
    expect(program.programId.toBase58()).to.equal(
      "4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B",
    );
  });

  it("initialize succeeds on local validator", async () => {
    const tx = await program.methods.initialize().rpc();
    expect(tx).to.be.a("string").with.length.greaterThan(0);
  });
});
