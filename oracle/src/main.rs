//! Entrypoint del Oracle de Autorización.

use anyhow::Context;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use oracle_authorization::{build_app, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env().context("error cargando configuración")?;
    let listen_addr = config.listen_addr();

    let app = build_app(config);
    let listener = TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("no se pudo bind en {listen_addr}"))?;

    tracing::info!(%listen_addr, "Oracle de autorización iniciado");

    axum::serve(listener, app)
        .await
        .context("error en el servidor HTTP")?;

    Ok(())
}
