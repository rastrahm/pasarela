# payment-settlement (Anchor)

Programa Solana del riel **SolanaWallet** — transferencia SPL atómica y evento `PaymentProcessed` sin PII.

> Fase 3 del [Plan de Implementación](../../Doc/Plan-de-Implementacion.md).  
> Directivas: [`solana.cursorrules`](../../solana.cursorrules)

## Requisitos

- Anchor CLI `0.31.1` (`avm install 0.31.1 && avm use 0.31.1`)
- Solana CLI ≥ 2.x
- **Platform-tools `v1.52`** (rustc 1.89) — el default de Solana 2.2.x (`v1.48`, rustc 1.84) no compila el lockfile actual por crates con `edition2024`
- Node.js 18+ + `npm`

### Toolchain Solana (importante)

Solana CLI 2.2.20 trae `platform-tools v1.48` (rustc 1.84). Para compilar:

```bash
# Descarga v1.52 en ~/.cache/solana/ (automático la primera vez)
cargo build-sbf --tools-version v1.52 --manifest-path programs/payment-settlement/Cargo.toml
```

El workspace declara `[workspace.metadata.solana] tools-version = "v1.52"` en `Cargo.toml` (requiere Agave ≥ 3.x para que `anchor build` lo respete automáticamente). Con Solana 2.2.x usar el script `npm run build`.

## Estructura

```
programs/payment-settlement/
├── Anchor.toml
├── Cargo.toml                 # workspace Rust del programa
├── package.json               # tests TypeScript (Mocha + ts-mocha)
├── programs/payment-settlement/
│   └── src/lib.rs             # instrucciones on-chain
├── tests/payment-settlement.ts
└── migrations/
```

## Comandos

```bash
cd programs/payment-settlement
npm install
npm test                 # build + anchor test (17 tests, validador local)
npm run test:ci          # mismo gate que CI (scripts/ci-test.sh)
```

### CI (gate 3.8)

Workflow: [`.github/workflows/programs-anchor-test.yml`](../../.github/workflows/programs-anchor-test.yml)

| Trigger | Job |
|---------|-----|
| PR / push (`programs/payment-settlement/**`) | Solana 2.2.20 + Anchor 0.31.1 + platform-tools v1.52 + `anchor test` |

Reproducir localmente el mismo gate:

```bash
./scripts/ci-test.sh
```

## Clusters (D2)

| Entorno | Uso |
|---------|-----|
| `localnet` | Desarrollo y `anchor test` |
| `devnet` | CI / staging (paso 3.9 opcional) |

Program ID (local/devnet scaffold): `4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B`

## Próximos pasos (Plan §7)

### Estado tests (3.8)

`npm test` / `anchor test --skip-build` — **17/17 verdes** en validador local.

| Paso | Entregable |
|------|------------|
| 3.9 | Devnet opcional |
| Gate Fase 3 | Acta de cierre + confirmación Fase 4 |

### PDA `SettlementState` (3.3)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `merchant` | Pubkey | Comercio (seed) |
| `total_amount` | u64 | Suma liquidada |
| `payment_count` | u64 | Pagos exitosos |
| `bump` | u8 | Bump canonical |

Seeds: `["settlement", merchant.as_ref()]`. Tests: `tests/settlement-state.ts`.

## Seguridad

- Zero PII on-chain (D6 / UC-07)
- `checked_add` / `checked_sub` para overflow
- Constraints explícitos en `#[derive(Accounts)]`
- Commitment `finalized` en Gateway (D10) — fuera de este crate
