//! Entrypoint del Oracle de Autorización.

use anyhow::Context;
use std::sync::Arc;
use tokio::net::TcpListener;

use oracle_authorization::{
    build_app, config::AppConfig, init_database, logging, persistence::AppState, ttl,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    logging::init_subscriber();

    let config = Arc::new(AppConfig::from_env().context("error cargando configuración")?);
    let listen_addr = config.listen_addr();

    let pool = init_database(&config.database_url)
        .await
        .context("error conectando a PostgreSQL")?;

    let state = AppState::new(config.clone(), pool)
        .map_err(|err| anyhow::anyhow!("error inicializando clientes: {err}"))?;

    ttl::spawn_ttl_cleanup_task(
        state.hold_store.clone(),
        state.config.ttl_cleanup_interval_secs,
    );

    let app = build_app(state);
    let listener = TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("no se pudo bind en {listen_addr}"))?;

    tracing::info!(%listen_addr, "Oracle de autorización iniciado");

    axum::serve(listener, app)
        .await
        .context("error en el servidor HTTP")?;

    Ok(())
}
