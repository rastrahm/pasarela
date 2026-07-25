//! Contrato HTTP público del Oracle — DTOs compartidos con futuro `oracle-client`.

pub mod dto;

pub use dto::{
    AuthorizeRequest, AuthorizeResponse, ErrorResponse, HealthResponse, ReleaseHoldRequest,
    ReleaseHoldResponse,
};
