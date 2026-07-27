> **Documentation / Documentación:** [Español (es)](README-es.md) · [English (en)](README-en.md)
>
# domain

Capa de dominio compartida de la pasarela multi-rail — **Fase 1** (tipos, traits y errores sin infraestructura).

> Crate puro: **sin** HTTP, PostgreSQL, Solana RPC ni clientes externos. Define el vocabulario común entre `rail-switcher`, el API Gateway y los adaptadores de liquidación.

## Responsabilidad

Centraliza los conceptos del negocio de pagos:

- Rieles de fondeo / liquidación (`FundingType`)
- Solicitud y respuesta de checkout (`PaymentRequest`, `PaymentResponse`)
- Ciclo de vida de transacciones (`TransactionStatus`)
- Holds off-chain y comprobantes (`HoldId`, `SettlementReceipt`)
- Traits de extensión (`PaymentProcessor`, `LiquidityEngine`)
- Errores tipados (`PaymentError`, `LiquidityError`, `RailError`)

## Módulos

| Módulo | Tipos principales |
|--------|-------------------|
| `card` | `CardNumber`, `CardPayload` — PAN redactado en `Debug` |
| `funding` | `FundingType`, `HoldId`, `FundStatus`, `SettlementReceipt` |
| `ids` | `TransactionId`, `MerchantId`, `Amount`, `Currency` |
| `payment` | `PaymentRequest`, `PaymentResponse`, `TransactionStatus` |
| `traits` | `PaymentProcessor`, `LiquidityEngine` |
| `error` | `PaymentError`, `LiquidityError`, `RailError` |

## Rieles (`FundingType`)

| Variante | Descripción |
|----------|-------------|
| `TraditionalBank` | Compensación bancaria simulada (ISO 20022 / ACH) |
| `BinanceCex` | Saldo custodial en exchange simulado |
| `SolanaWallet` | Liquidación SPL on-chain (Anchor) |

Serialización JSON: `snake_case` (`traditional_bank`, `binance_cex`, `solana_wallet`).

## Traits (Strategy)

```rust
pub trait PaymentProcessor {
    fn process(&self, request: PaymentRequest) -> Result<PaymentResponse, PaymentError>;
}

pub trait LiquidityEngine {
    fn evaluate_funds(&self, amount: Amount, rail: FundingType) -> Result<FundStatus, LiquidityError>;
    fn hold(&self, amount: Amount) -> Result<HoldId, LiquidityError>;
    fn settle(&self, hold_id: HoldId) -> Result<SettlementReceipt, LiquidityError>;
}
```

Implementaciones concretas viven fuera de este crate (Oracle, adaptadores de settlement, mocks en tests).

## Uso

```rust
use domain::{
    Amount, CardNumber, CardPayload, Currency, FundingType, MerchantId,
    PaymentRequest, TransactionId,
};

let request = PaymentRequest {
    transaction_id: TransactionId::generate(),
    merchant_id: MerchantId::new(uuid::Uuid::new_v4()),
    amount: Amount::from_units(100),
    currency: Currency::new("USD").expect("ISO 4217"),
    card: CardPayload {
        pan: CardNumber::new("4111111111111111"),
        expiry_month: "12".into(),
        expiry_year: "2030".into(),
        cvv: "123".into(),
        cardholder: "Test User".into(),
    },
    funding_type: Some(FundingType::TraditionalBank),
};
```

## Consumidores

| Crate | Uso |
|-------|-----|
| [`rail-switcher`](../rail-switcher/) | Selección y fallback entre rieles |
| [`settlement-adapters`](../settlement-adapters/) | Liquidación por riel |
| [`api-gateway`](../api-gateway/) | Orquestación checkout, persistencia, errores HTTP |

> **Nota:** el Oracle (`oracle/`) usa DTOs propios vía `oracle-client`; no depende de `domain` para el contrato wire.

## Seguridad

- `CardNumber` y `CardPayload` ocultan PAN/CVV en logs (`Debug` redactado).
- Validación Luhn y antifraude ocurren en capas externas (Oracle), no en este crate.

## Tests

```bash
cargo test -p domain
```

## Referencias

- [Doc/Arquitectura.md](../../Doc/Arquitectura.md) — decisiones D1–D12, fronteras de dominio
- [Doc/Casos-de-Uso-ER-Flujos.md](../../Doc/Casos-de-Uso-ER-Flujos.md) — UC-01–UC-11
