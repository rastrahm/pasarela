//! Cliente HTTP tipado hacia el Oracle de autorización (contrato API v1).
//!
//! Consumido exclusivamente por el API Gateway. Define DTOs compartidos y
//! [`HttpOracleClient`] para invocar `/internal/v1/*`.

mod client;
mod dto;
mod error;
mod http;

pub use client::{OracleClient, RequestOptions};
pub use dto::{
    AuthorizeRequest, AuthorizeResponse, CardPayload, ErrorResponse, FundingType, HealthResponse,
    ReleaseHoldRequest, ReleaseHoldResponse,
};
pub use dto::error_codes;
pub use error::OracleClientError;
pub use http::HttpOracleClient;
