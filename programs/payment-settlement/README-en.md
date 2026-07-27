# payment-settlement (Anchor)

Solana program for the **SolanaWallet** rail — atomic SPL transfer and `PaymentProcessed` event without PII.

> Phase 3 of the [Implementation Plan](../../Doc/Plan-de-Implementacion-en.md).  
> Directives: [`solana.cursorrules`](../../solana.cursorrules)

## Requirements

- Anchor CLI `0.31.1` (`avm install 0.31.1 && avm use 0.31.1`)
- Solana CLI ≥ 2.x
- **Platform-tools `v1.52`** (rustc 1.89) — the Solana 2.2.x default (`v1.48`, rustc 1.84) does not compile the current lockfile due to crates with `edition2024`
- Node.js 18+ + `npm`

### Solana toolchain (important)

Solana CLI 2.2.20 ships `platform-tools v1.48` (rustc 1.84). To compile:

```bash
# Downloads v1.52 to ~/.cache/solana/ (automatic the first time)
cargo build-sbf --tools-version v1.52 --manifest-path programs/payment-settlement/Cargo.toml
```

The workspace declares `[workspace.metadata.solana] tools-version = "v1.52"` in `Cargo.toml` (requires Agave ≥ 3.x for `anchor build` to respect it automatically). With Solana 2.2.x use the `npm run build` script.

## Structure

```
programs/payment-settlement/
├── Anchor.toml
├── Cargo.toml                 # program Rust workspace
├── package.json               # TypeScript tests (Mocha + ts-mocha)
├── programs/payment-settlement/
│   └── src/lib.rs             # on-chain instructions
├── tests/payment-settlement.ts
└── migrations/
```

## Commands

```bash
cd programs/payment-settlement
npm install
npm test                 # build + anchor test (17 tests, local validator)
npm run test:ci          # gate 3.8 + 3.10 (scripts/ci-test.sh)
npm run lint:docs        # verifies @notice/@param/@return on instructions
```

### CI (gate 3.8)

Workflow: [`.github/workflows/programs-anchor-test.yml`](../../.github/workflows/programs-anchor-test.yml)

| Trigger | Job |
|---------|-----|
| PR / push (`programs/payment-settlement/**`) | Solana 2.2.20 + Anchor 0.31.1 + platform-tools v1.52 + `anchor test` |

Reproduce the same gate locally:

```bash
./scripts/ci-test.sh
```

## Clusters (D2)

| Environment | Purpose | RPC |
|---------|-----|-----|
| `localnet` | Development and `anchor test` | `http://127.0.0.1:8899` |
| `devnet` | Staging / Gateway integration (3.9) | `https://api.devnet.solana.com` |

Program ID (localnet + devnet): `4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B`

### Devnet (step 3.9)

Deployed on **2026-07-25**. Metadata in [`deploy/devnet.json`](deploy/devnet.json).

```bash
npm run deploy:devnet    # build + anchor deploy --provider.cluster devnet
npm run verify:devnet    # solana program show against devnet
```

Environment variables: [`.env.example`](.env.example) (`SOLANA_RPC_URL`, `PAYMENT_SETTLEMENT_PROGRAM_ID`).

## Next steps (Plan §7)

### Test status (3.8)

`npm test` / `anchor test --skip-build` — **17/17 green** on local validator.

| Step | Deliverable |
|------|------------|
| 3.9 | ✅ Devnet — `deploy/devnet.json` |
| 3.10 | ✅ `@notice/@param/@return` — gate `npm run lint:docs` |
| Phase 3 gate | Closure record pending; Anchor technical gate ✅ |
| Phase 4 gate | [Acta-Cierre-Fase-4-en.md](../../Doc/Acta-Cierre-Fase-4-en.md) (2026-07-26) |

### PDA `SettlementState` (3.3)

| Field | Type | Description |
|-------|------|-------------|
| `merchant` | Pubkey | Merchant (seed) |
| `total_amount` | u64 | Settled sum |
| `payment_count` | u64 | Successful payments |
| `bump` | u8 | Canonical bump |

Seeds: `["settlement", merchant.as_ref()]`. Tests: `tests/settlement-state.ts`.

## Security

- Zero PII on-chain (D6 / UC-07)
- `checked_add` / `checked_sub` for overflow
- Explicit constraints in `#[derive(Accounts)]`
- Commitment `finalized` in Gateway (D10) — outside this crate
