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
npm run build          # SBF (v1.52) + IDL + types TS
anchor test --skip-build   # levanta validador local + 2 tests bootstrap
# atajo:
npm run build && anchor test --skip-build
```

## Clusters (D2)

| Entorno | Uso |
|---------|-----|
| `localnet` | Desarrollo y `anchor test` |
| `devnet` | CI / staging (paso 3.9 opcional) |

Program ID (local/devnet scaffold): `4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B`

## Próximos pasos (Plan §7)

| Paso | Entregable |
|------|------------|
| 3.4 | Instrucción `process_payment` — hacer verde los 4 tests §7.4 |
| 3.5 | Evento `PaymentProcessed` |
| 3.6 | Constraints explícitos restantes en `#[derive(Accounts)]` |
| 3.8 | `anchor test` verde |

### PDA `SettlementState` (3.3)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `merchant` | Pubkey | Comercio (seed) |
| `total_amount` | u64 | Suma liquidada |
| `payment_count` | u64 | Pagos exitosos |
| `bump` | u8 | Bump canonical |

Seeds: `["settlement", merchant.as_ref()]`. Tests: `tests/settlement-state.ts`.

### Estado TDD (3.2)

Los tests en `tests/process-payment.ts` están escritos **antes** de la lógica. Hasta 3.4–3.6 fallan con `NotImplemented` o errores de constraint pendientes:

| Test §7.4 | Error actual (esperado) |
|-----------|------------------------|
| Firma no autorizada | `NotImplemented` → `Unauthorized` (3.6) |
| Espacio insuficiente | `NotImplemented` → `InsufficientAccountSpace` (3.6) |
| Overflow aritmético | `NotImplemented` → `AmountOverflow` (3.4) |
| Transferencia OK + evento | `NotImplemented` → éxito (3.4–3.5) |

Bootstrap (3.1): 2 tests verdes. PDA seeds: 1 test verde.

## Seguridad

- Zero PII on-chain (D6 / UC-07)
- `checked_add` / `checked_sub` para overflow
- Constraints explícitos en `#[derive(Accounts)]`
- Commitment `finalized` en Gateway (D10) — fuera de este crate
