//! Entrypoint del API Gateway.

use anyhow::Context;
use std::sync::Arc;
use tokio::net::TcpListener;

use api_gateway::{build_app, config::AppConfig, logging, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    logging::init_subscriber();

    let config = Arc::new(AppConfig::from_env().context("error cargando configuración")?);
    let listen_addr = config.listen_addr();

    let merchant_registry = Arc::new(
        config
            .build_merchant_registry()
            .context("error cargando API keys de comercio")?,
    );
    let state = AppState::new(config, merchant_registry)
        .await
        .context("error inicializando estado del Gateway (Oracle inalcanzable?)")?;
    let app = build_app(state);

    let listener = TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("no se pudo bind en {listen_addr}"))?;

    tracing::info!(%listen_addr, "API Gateway iniciado");

    axum::serve(listener, app)
        .await
        .context("error en el servidor HTTP")?;

    Ok(())
}
