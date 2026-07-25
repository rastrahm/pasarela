import { AnchorError } from "@coral-xyz/anchor";
import { expect } from "chai";

/** Asserts an Anchor instruction failed with the expected custom error code. */
export function expectAnchorError(err: unknown, code: string): void {
  expect(err).to.exist;
  const anchorErr = err as AnchorError;
  expect(anchorErr.error?.errorCode?.code, JSON.stringify(anchorErr)).to.equal(
    code,
  );
}

/** Asserts the transaction succeeded (guards accidental `.rpc()` without rejection). */
export function expectTxSuccess(signature: string): void {
  expect(signature).to.be.a("string").with.length.greaterThan(0);
}
