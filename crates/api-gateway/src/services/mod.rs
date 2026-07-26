//! Servicios de aplicación del Gateway.

pub mod checkout;
mod rails;

pub use checkout::{extract_caller_ip, process_checkout, CheckoutInput};
