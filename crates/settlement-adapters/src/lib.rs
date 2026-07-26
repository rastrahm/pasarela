//! Adaptadores de liquidación por riel — Strategy pattern.
//!
//! Consumido por el API Gateway (Fase 4). Cada riel implementa [`SettlementAdapter`]
//! y el [`SettlementEngine`] despacha según el [`domain::FundingType`] activo.

mod adapter;
mod bank;
mod binance;
mod context;
mod engine;
mod mock;
mod solana;

pub use adapter::SettlementAdapter;
pub use bank::{BankCompensation, Pacs008Document, TraditionalBankAdapter, generate_bank_compensation};
pub use binance::{
    BinanceCexAdapter, BinanceCexConfig, BinanceClientError, BinanceConfigError, BinanceSpotClient,
    HttpBinanceSpotClient, InMemoryBinanceSpotClient, SpotDebitRequest, SpotDebitResponse,
};
pub use context::SettlementContext;
pub use engine::SettlementEngine;
pub use mock::MockSettlementAdapter;
pub use solana::{
    InMemorySolanaSettlementClient, ProcessPaymentAccounts, ProcessPaymentRequest,
    ProcessPaymentResult, RpcSolanaSettlementClient, SolanaClientError, SolanaConfigError,
    SolanaSettlementClient, SolanaSettlementConfig, SolanaWalletAdapter,
    amount_to_base_units, build_process_payment_instruction, derive_process_payment_accounts,
    process_payment_discriminator,
};
