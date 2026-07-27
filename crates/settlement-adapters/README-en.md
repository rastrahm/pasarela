# settlement-adapters

Settlement adapters per rail — **Strategy pattern** (Phase 4 ✅).

> Consumer: **API Gateway** (`crates/api-gateway/`). The Oracle manages holds; these adapters execute settlement on the active rail.

## Rails

| Adapter | Rail | Settlement proof | Step |
|-----------|------|------------------------|------|
| `TraditionalBankAdapter` | `TraditionalBank` | Bank reference (ISO 20022 / ACH) | ✅ 4.2 |
| `BinanceCexAdapter` | `BinanceCex` | CEX order ID | ✅ 4.3 |
| `SolanaWalletAdapter` | `SolanaWallet` | On-chain tx signature | ✅ 4.4 |

## Usage

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
