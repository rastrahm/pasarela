//! Adapters de consulta de saldo por riel (Plan 2.6, UC-04).

mod client;
mod composite;
mod config_bank;
mod error;
mod http_binance;
mod mock;
mod rpc_solana;

pub use client::RailBalanceProvider;
pub use composite::CompositeRailProvider;
pub use error::RailAdapterError;
pub use mock::MockRailBalanceProvider;
