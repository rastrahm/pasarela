# domain

Shared domain layer for the multi-rail payment gateway — **Phase 1** (types, traits, and errors with no infrastructure).

> Pure crate: **no** HTTP, PostgreSQL, Solana RPC, or external clients. Defines the common vocabulary for `rail-switcher`, the API Gateway, and settlement adapters.

## Responsibility

Centralizes payment business concepts:

- Funding / settlement rails (`FundingType`)
- Checkout request and response (`PaymentRequest`, `PaymentResponse`)
- Transaction lifecycle (`TransactionStatus`)
- Off-chain holds and receipts (`HoldId`, `SettlementReceipt`)
- Extension traits (`PaymentProcessor`, `LiquidityEngine`)
- Typed errors (`PaymentError`, `LiquidityError`, `RailError`)

## Modules

| Module | Main types |
|--------|------------|
| `card` | `CardNumber`, `CardPayload` — PAN redacted in `Debug` |
| `funding` | `FundingType`, `HoldId`, `FundStatus`, `SettlementReceipt` |
| `ids` | `TransactionId`, `MerchantId`, `Amount`, `Currency` |
| `payment` | `PaymentRequest`, `PaymentResponse`, `TransactionStatus` |
| `traits` | `PaymentProcessor`, `LiquidityEngine` |
| `error` | `PaymentError`, `LiquidityError`, `RailError` |

## Rails (`FundingType`)

| Variant | Description |
|---------|-------------|
| `TraditionalBank` | Simulated bank settlement (ISO 20022 / ACH) |
| `BinanceCex` | Custodial balance on simulated exchange |
| `SolanaWallet` | On-chain SPL settlement (Anchor) |

JSON serialization: `snake_case` (`traditional_bank`, `binance_cex`, `solana_wallet`).

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

Concrete implementations live outside this crate (Oracle, settlement adapters, test mocks).

## Usage

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

## Consumers

| Crate | Usage |
|-------|-------|
| [`rail-switcher`](../rail-switcher/) | Rail selection and fallback |
| [`settlement-adapters`](../settlement-adapters/) | Per-rail settlement |
| [`api-gateway`](../api-gateway/) | Checkout orchestration, persistence, HTTP errors |

> **Note:** the Oracle (`oracle/`) uses its own DTOs via `oracle-client`; it does not depend on `domain` for the wire contract.

## Security

- `CardNumber` and `CardPayload` hide PAN/CVV in logs (redacted `Debug`).
- Luhn validation and antifraud run in external layers (Oracle), not in this crate.

## Tests

```bash
cargo test -p domain
```

## References

- [Doc/Arquitectura-en.md](../../Doc/Arquitectura-en.md) — decisions D1–D12, domain boundaries
- [Doc/Casos-de-Uso-ER-Flujos-en.md](../../Doc/Casos-de-Uso-ER-Flujos-en.md) — UC-01–UC-11
