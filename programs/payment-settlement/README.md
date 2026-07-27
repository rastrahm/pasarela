> **Documentation / Documentación:** [Español (es)](README-es.md) · [English (en)](README-en.md)
>
# payment-settlement (Anchor)

Programa Solana del riel **SolanaWallet** — **Fase 3** (liquidación SPL on-chain).

> Transferencia atómica de tokens SPL al comercio, PDA acumuladora por merchant y evento `PaymentProcessed` **sin PII** (D6 / UC-07).  
> Directivas: [`solana.cursorrules`](../../solana.cursorrules)

## Responsabilidad

| Aspecto | Detalle |
|---------|---------|
| Riel | `SolanaWallet` (`FundingType` en dominio) |
| Instrucción principal | `process_payment` — debita SPL del pagador, acredita al comercio |
| Estado on-chain | PDA `SettlementState` por comercio (contadores acumulados) |
| Auditoría | Evento `PaymentProcessed { amount, brand_code, settlement_rail_id, timestamp }` |
| Fuera de alcance | Autorización off-chain, holds, antifraude — Oracle + Gateway |

## Instrucciones on-chain

| Instrucción | Uso |
|-------------|-----|
| `initialize` | Smoke post-deploy (no-op) |
| `initialize_settlement` | Crea PDA `SettlementState` para un merchant |
| `process_payment` | Transferencia SPL + actualización de contadores + evento |
| `set_settlement_totals` | **Solo tests** — ajuste de contadores TDD |
| `initialize_settlement_undersized` | **Solo tests** — fixture de espacio insuficiente |

## PDA `SettlementState`

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `merchant` | Pubkey | Comercio (seed) |
| `total_amount` | u64 | Suma liquidada (base units SPL) |
| `payment_count` | u64 | Pagos exitosos |
| `bump` | u8 | Bump canonical |

Seeds: `["settlement", merchant.as_ref()]`. Tests: `tests/settlement-state.ts`.

## Integración

| Componente | Rol |
|------------|-----|
| [`settlement-adapters`](../../crates/settlement-adapters/) | `SolanaWalletAdapter` construye y envía la tx `process_payment` |
| [`api-gateway`](../../crates/api-gateway/) | Orquesta checkout; usa el adapter vía `SettlementEngine` |
| Oracle | **No** invoca el programa — solo evalúa saldo SPL off-chain |

Program ID (localnet + devnet): `4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B`

## Requisitos

- Anchor CLI `0.31.1` (`avm install 0.31.1 && avm use 0.31.1`)
- Solana CLI ≥ 2.x
- **Platform-tools `v1.52`** (rustc 1.89) — el default de Solana 2.2.x (`v1.48`) no compila el lockfile actual
- Node.js 18+ + `npm`

### Toolchain Solana (importante)

```bash
# Descarga v1.52 en ~/.cache/solana/ (automático la primera vez)
cargo build-sbf --tools-version v1.52 --manifest-path programs/payment-settlement/Cargo.toml
```

El workspace declara `tools-version = "v1.52"` en `Cargo.toml`. Con Solana 2.2.x usar `npm run build`.

## Estructura

```
programs/payment-settlement/
├── Anchor.toml
├── Cargo.toml
├── package.json
├── programs/payment-settlement/src/
│   ├── lib.rs              # instrucciones
│   ├── state.rs            # SettlementState, PaymentProcessed
│   └── constraints.rs      # validaciones PDA
├── tests/                  # Mocha + ts-mocha (17 tests)
├── deploy/devnet.json      # metadatos deploy devnet
└── scripts/ci-test.sh
```

## Comandos

```bash
cd programs/payment-settlement
npm install
npm test                 # build + anchor test (17 tests, validador local)
npm run test:ci          # gate CI (scripts/ci-test.sh)
npm run lint:docs        # @notice/@param/@return en instrucciones
npm run deploy:devnet    # build + anchor deploy --provider.cluster devnet
npm run verify:devnet    # solana program show contra devnet
```

Variables: [`.env.example`](.env.example) (`SOLANA_RPC_URL`, `PAYMENT_SETTLEMENT_PROGRAM_ID`).

## Clusters (D2)

| Entorno | Uso | RPC |
|---------|-----|-----|
| `localnet` | Desarrollo y `anchor test` | `http://127.0.0.1:8899` |
| `devnet` | Staging / integración Gateway | `https://api.devnet.solana.com` |

Devnet desplegado **2026-07-25** — metadatos en [`deploy/devnet.json`](deploy/devnet.json).

## CI

Workflow: [`.github/workflows/programs-anchor-test.yml`](../../.github/workflows/programs-anchor-test.yml)

| Trigger | Job |
|---------|-----|
| PR / push (`programs/payment-settlement/**`) | Solana 2.2.20 + Anchor 0.31.1 + platform-tools v1.52 + `anchor test` |

## Estado (Plan §7)

| Paso | Entregable |
|------|------------|
| 3.8 | ✅ 17/17 tests verdes en validador local |
| 3.9 | ✅ Devnet — `deploy/devnet.json` |
| 3.10 | ✅ Documentación instrucciones — `npm run lint:docs` |
| Gate Fase 4 | [Acta-Cierre-Fase-4.md](../../Doc/Acta-Cierre-Fase-4.md) (2026-07-26) |

## Seguridad

- Zero PII on-chain (D6 / UC-07) — solo `brand_code` numérico, no PAN
- `checked_add` en contadores PDA (`AmountOverflow`)
- Constraints explícitos en `ProcessPayment` (owner, mint, saldo, espacio PDA)
- Commitment `finalized` en Gateway (D10) — fuera de este programa

## Referencias

- [Doc/Arquitectura.md](../../Doc/Arquitectura.md) — §7 on-chain, D2, D6, D10
- [Doc/Casos-de-Uso-ER-Flujos.md](../../Doc/Casos-de-Uso-ER-Flujos.md) — UC-07
- [Doc/Plan-de-Implementacion.md](../../Doc/Plan-de-Implementacion.md) — Fase 3
- [settlement-adapters/README.md](../../crates/settlement-adapters/README.md) — adapter Solana
