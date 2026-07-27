> **Documentation / Documentación:** [Español (es)](README-es.md) · [English (en)](README-en.md)
>
# settlement-adapters

Adaptadores de liquidación por riel — **Strategy pattern** (Fase 4 ✅).

> Consumidor: **API Gateway** (`crates/api-gateway/`). El Oracle gestiona holds; estos adaptadores ejecutan el asentamiento en el riel activo.

## Rieles

| Adaptador | Riel | Prueba de asentamiento | Paso |
|-----------|------|------------------------|------|
| `TraditionalBankAdapter` | `TraditionalBank` | Referencia bancaria (ISO 20022 / ACH) | ✅ 4.2 |
| `BinanceCexAdapter` | `BinanceCex` | ID de orden CEX | ✅ 4.3 |
| `SolanaWalletAdapter` | `SolanaWallet` | Tx signature on-chain | ✅ 4.4 |

## Uso

```rust
use settlement_adapters::{SettlementEngine, SettlementContext, MockSettlementAdapter};
use domain::FundingType;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), domain::LiquidityError> {
    let engine = SettlementEngine::new([
        Arc::new(MockSettlementAdapter::new(
            FundingType::TraditionalBank,
            "ACH-20260726-001",
        )),
    ]);

    let receipt = engine
        .settle(FundingType::TraditionalBank, context)
        .await?;

    println!("proof={}", receipt.proof);
    Ok(())
}
```

## Tests

```bash
cargo test -p settlement-adapters
```
