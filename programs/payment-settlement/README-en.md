# payment-settlement (Anchor)

Solana program for the **SolanaWallet** rail — **Phase 3** (on-chain SPL settlement).

> Atomic SPL token transfer to the merchant, per-merchant accumulator PDA, and `PaymentProcessed` event **without PII** (D6 / UC-07).  
> Directives: [`solana.cursorrules`](../../solana.cursorrules)

## Responsibility

| Aspect | Detail |
|--------|--------|
| Rail | `SolanaWallet` (`FundingType` in domain) |
| Main instruction | `process_payment` — debits payer SPL, credits merchant |
| On-chain state | `SettlementState` PDA per merchant (accumulated counters) |
| Audit | Event `PaymentProcessed { amount, brand_code, settlement_rail_id, timestamp }` |
| Out of scope | Off-chain authorization, holds, antifraud — Oracle + Gateway |

## On-chain instructions

| Instruction | Purpose |
|-------------|---------|
| `initialize` | Post-deploy smoke (no-op) |
| `initialize_settlement` | Creates `SettlementState` PDA for a merchant |
| `process_payment` | SPL transfer + counter update + event emission |
| `set_settlement_totals` | **Tests only** — TDD counter adjustment |
| `initialize_settlement_undersized` | **Tests only** — insufficient space fixture |

## PDA `SettlementState`

| Field | Type | Description |
|-------|------|-------------|
| `merchant` | Pubkey | Merchant (seed) |
| `total_amount` | u64 | Settled sum (SPL base units) |
| `payment_count` | u64 | Successful payments |
| `bump` | u8 | Canonical bump |

Seeds: `["settlement", merchant.as_ref()]`. Tests: `tests/settlement-state.ts`.

## Integration

| Component | Role |
|-----------|------|
| [`settlement-adapters`](../../crates/settlement-adapters/) | `SolanaWalletAdapter` builds and sends the `process_payment` tx |
| [`api-gateway`](../../crates/api-gateway/) | Orchestrates checkout; uses the adapter via `SettlementEngine` |
| Oracle | Does **not** invoke the program — only evaluates SPL balance off-chain |

Program ID (localnet + devnet): `4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B`

## Requirements

- Anchor CLI `0.31.1` (`avm install 0.31.1 && avm use 0.31.1`)
- Solana CLI ≥ 2.x
- **Platform-tools `v1.52`** (rustc 1.89) — Solana 2.2.x default (`v1.48`) does not compile the current lockfile
- Node.js 18+ + `npm`

### Solana toolchain (important)

```bash
# Downloads v1.52 to ~/.cache/solana/ (automatic the first time)
cargo build-sbf --tools-version v1.52 --manifest-path programs/payment-settlement/Cargo.toml
```

The workspace declares `tools-version = "v1.52"` in `Cargo.toml`. With Solana 2.2.x use `npm run build`.

## Structure

```
programs/payment-settlement/
├── Anchor.toml
├── Cargo.toml
├── package.json
├── programs/payment-settlement/src/
│   ├── lib.rs              # instructions
│   ├── state.rs            # SettlementState, PaymentProcessed
│   └── constraints.rs      # PDA validations
├── tests/                  # Mocha + ts-mocha (17 tests)
├── deploy/devnet.json      # devnet deploy metadata
└── scripts/ci-test.sh
```

## Commands

```bash
cd programs/payment-settlement
npm install
npm test                 # build + anchor test (17 tests, local validator)
npm run test:ci          # CI gate (scripts/ci-test.sh)
npm run lint:docs        # @notice/@param/@return on instructions
npm run deploy:devnet    # build + anchor deploy --provider.cluster devnet
npm run verify:devnet    # solana program show against devnet
```

Environment: [`.env.example`](.env.example) (`SOLANA_RPC_URL`, `PAYMENT_SETTLEMENT_PROGRAM_ID`).

## Clusters (D2)

| Environment | Purpose | RPC |
|-------------|---------|-----|
| `localnet` | Development and `anchor test` | `http://127.0.0.1:8899` |
| `devnet` | Staging / Gateway integration | `https://api.devnet.solana.com` |

Devnet deployed **2026-07-25** — metadata in [`deploy/devnet.json`](deploy/devnet.json).

## CI

Workflow: [`.github/workflows/programs-anchor-test.yml`](../../.github/workflows/programs-anchor-test.yml)

| Trigger | Job |
|---------|-----|
| PR / push (`programs/payment-settlement/**`) | Solana 2.2.20 + Anchor 0.31.1 + platform-tools v1.52 + `anchor test` |

## Status (Plan §7)

| Step | Deliverable |
|------|-------------|
| 3.8 | ✅ 17/17 tests green on local validator |
| 3.9 | ✅ Devnet — `deploy/devnet.json` |
| 3.10 | ✅ Instruction docs — `npm run lint:docs` |
| Phase 4 gate | [Acta-Cierre-Fase-4-en.md](../../Doc/Acta-Cierre-Fase-4-en.md) (2026-07-26) |

## Security

- Zero PII on-chain (D6 / UC-07) — numeric `brand_code` only, no PAN
- `checked_add` on PDA counters (`AmountOverflow`)
- Explicit constraints on `ProcessPayment` (owner, mint, balance, PDA space)
- Commitment `finalized` in Gateway (D10) — outside this program

## References

- [Doc/Arquitectura-en.md](../../Doc/Arquitectura-en.md) — §7 on-chain, D2, D6, D10
- [Doc/Casos-de-Uso-ER-Flujos-en.md](../../Doc/Casos-de-Uso-ER-Flujos-en.md) — UC-07
- [Doc/Plan-de-Implementacion-en.md](../../Doc/Plan-de-Implementacion-en.md) — Phase 3
- [settlement-adapters/README-en.md](../../crates/settlement-adapters/README-en.md) — Solana adapter
