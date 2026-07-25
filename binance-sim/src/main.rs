//! Entrypoint del simulador Binance Spot.

use anyhow::Context;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use binance_sim_service::{build_app, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Arc::new(AppConfig::from_env().context("error cargando configuración")?);
    let listen_addr = config.listen_addr();

    let app = build_app((*config).clone());
    let listener = TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("no se pudo bind en {listen_addr}"))?;

    tracing::info!(%listen_addr, "Simulador Binance Spot iniciado");

    axum::serve(listener, app)
        .await
        .context("error en el servidor HTTP")?;

    Ok(())
}
