//! Contrato HTTP del Oracle — re-exportado desde `oracle-client` (fuente de verdad del wire format).

pub use oracle_client::{
    error_codes, AuthorizeRequest, AuthorizeResponse, CardPayload, ErrorResponse, HealthResponse,
    ReleaseHoldRequest, ReleaseHoldResponse,
};
