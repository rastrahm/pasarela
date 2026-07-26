//! Servicios de aplicación del Gateway.

pub mod checkout;
pub mod rails;

pub use checkout::{extract_caller_ip, process_checkout, CheckoutInput};
pub use rails::RailContext;
