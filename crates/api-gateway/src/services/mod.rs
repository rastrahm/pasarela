//! Servicios de aplicación del Gateway.

pub mod checkout;
pub mod idempotency;
pub mod rails;
pub mod transactions;

pub use checkout::{extract_caller_ip, process_checkout, CheckoutInput};
pub use idempotency::{
    extract_idempotency_key, process_checkout_idempotent, IDEMPOTENCY_KEY_HEADER,
};
pub use rails::RailContext;
pub use transactions::lookup_transaction;
