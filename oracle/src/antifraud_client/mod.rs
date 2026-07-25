//! Cliente HTTP tipado hacia el servicio antifraude (UC-12, D11).

mod client;
mod error;
mod http;
mod mock;
mod types;

pub use client::AntifraudClient;
pub use error::AntifraudClientError;
pub use http::HttpAntifraudClient;
pub use mock::{MockAntifraudClient, MockBehavior};
pub use types::{ScoreRequest, ScoreResponse};
