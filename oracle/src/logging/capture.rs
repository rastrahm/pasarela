//! Captura de logs para tests de ausencia de PII.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing_subscriber::EnvFilter;

struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

impl Write for SharedBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("lock log buffer").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Ejecuta un closure con un subscriber que captura la salida de tracing.
pub fn run_with_log_capture<F, R>(f: F) -> (R, String)
where
    F: FnOnce() -> R,
{
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = buffer.clone();

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || SharedBuffer(writer.clone()))
        .with_env_filter(EnvFilter::new("trace"))
        .finish();

    let result = tracing::subscriber::with_default(subscriber, f);
    let logs = String::from_utf8_lossy(&buffer.lock().expect("lock log buffer")).into_owned();
    (result, logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_records_tracing_output() {
        let (_, logs) = run_with_log_capture(|| {
            tracing::info!(event = "test_event", "mensaje de prueba");
        });

        assert!(logs.contains("test_event"));
        assert!(logs.contains("mensaje de prueba"));
    }
}
