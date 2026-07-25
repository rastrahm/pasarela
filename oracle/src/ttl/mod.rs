//! Tarea en background para expiración proactiva de holds (TTL híbrido).

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info};

use crate::persistence::hold_store::HoldStore;

/// Arranca la tarea periódica que expira holds vencidos.
///
/// # Inputs
/// - `hold_store`: store de holds.
/// - `interval_secs`: intervalo entre barridos en segundos.
pub fn spawn_ttl_cleanup_task(hold_store: Arc<dyn HoldStore>, interval_secs: u64) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval_secs);
        info!(interval_secs, "tarea TTL de holds iniciada");

        loop {
            tokio::time::sleep(interval).await;

            match hold_store.expire_stale_holds().await {
                Ok(count) if count > 0 => {
                    debug!(expired = count, "holds expirados por tarea background");
                }
                Ok(_) => {}
                Err(err) => {
                    error!(error = %err, "fallo al expirar holds en background");
                }
            }
        }
    });
}
