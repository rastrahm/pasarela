//! Capa de dominio compartida de la pasarela multi-rail.
//!
//! Define tipos, traits y errores sin dependencias de HTTP, base de datos ni Solana.
//! Consumido por `rail-switcher`, el API Gateway (Fase 4) y adaptadores de liquidación.

mod card;
mod error;
mod funding;
mod ids;
mod payment;
mod traits;

pub use card::{CardNumber, CardPayload};
pub use error::{LiquidityError, PaymentError, RailError};
pub use funding::{FundStatus, FundingType, HoldId, SettlementReceipt};
pub use ids::{Amount, Currency, MerchantId, TransactionId};
pub use payment::{PaymentRequest, PaymentResponse, TransactionStatus};
pub use traits::{LiquidityEngine, PaymentProcessor};
