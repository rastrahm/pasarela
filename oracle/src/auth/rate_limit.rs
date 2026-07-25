//! Rate limiting con ventana deslizante por clave compuesta (API key + IP).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::error::AppError;

const WINDOW_SECS: u64 = 60;

/// Límite de solicitudes por ventana deslizante de 60 segundos.
#[derive(Debug)]
pub struct SlidingWindowRateLimiter {
    max_requests: u32,
    window: Duration,
    entries: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SlidingWindowRateLimiter {
    /// Crea un limitador con tope de solicitudes por minuto.
    ///
    /// # Inputs
    /// - `max_requests`: máximo de solicitudes permitidas en la ventana de 60 s.
    ///
    /// # Returns
    /// Limitador listo para uso concurrente vía `Arc`.
    pub fn new(max_requests: u32) -> Arc<Self> {
        Arc::new(Self {
            max_requests,
            window: Duration::from_secs(WINDOW_SECS),
            entries: Mutex::new(HashMap::new()),
        })
    }

    /// Crea un limitador con ventana configurable (tests).
    pub fn with_window(max_requests: u32, window: Duration) -> Arc<Self> {
        Arc::new(Self {
            max_requests,
            window,
            entries: Mutex::new(HashMap::new()),
        })
    }

    /// Registra una solicitud si no se superó el límite.
    ///
    /// # Inputs
    /// - `key`: identificador compuesto (p. ej. `{api_key}:{ip}`).
    ///
    /// # Returns
    /// `Ok(())` si la solicitud está permitida; `TooManyRequests` si se excedió el cupo.
    pub async fn try_acquire(&self, key: &str) -> Result<(), AppError> {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();
        let window_start = now.checked_sub(self.window).unwrap_or(now);

        let bucket = entries.entry(key.to_string()).or_default();
        bucket.retain(|instant| *instant >= window_start);

        if bucket.len() as u32 >= self.max_requests {
            return Err(AppError::TooManyRequests);
        }

        bucket.push(now);
        Ok(())
    }
}

/// Construye la clave de rate limit a partir de API key e IP del caller.
pub fn rate_limit_key(api_key: &str, ip: IpAddrDisplay<'_>) -> String {
    format!("{}:{}", api_key, ip.0)
}

/// IP formateada para la clave de rate limit.
pub struct IpAddrDisplay<'a>(pub &'a str);

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn allows_requests_under_limit() {
        let limiter = SlidingWindowRateLimiter::with_window(3, Duration::from_secs(60));
        for _ in 0..3 {
            limiter
                .try_acquire("key:127.0.0.1")
                .await
                .expect("dentro del límite");
        }
    }

    #[tokio::test]
    async fn rejects_request_over_limit() {
        let limiter = SlidingWindowRateLimiter::with_window(2, Duration::from_secs(60));
        limiter.try_acquire("key:127.0.0.1").await.expect("first");
        limiter.try_acquire("key:127.0.0.1").await.expect("second");

        let result = limiter.try_acquire("key:127.0.0.1").await;
        assert!(matches!(result, Err(AppError::TooManyRequests)));
    }

    #[tokio::test]
    async fn separate_keys_have_independent_buckets() {
        let limiter = SlidingWindowRateLimiter::with_window(1, Duration::from_secs(60));
        limiter.try_acquire("key-a:127.0.0.1").await.expect("a");
        limiter.try_acquire("key-b:127.0.0.1").await.expect("b");

        let result = limiter.try_acquire("key-a:127.0.0.1").await;
        assert!(matches!(result, Err(AppError::TooManyRequests)));
    }

    #[tokio::test]
    async fn expired_requests_free_capacity() {
        let limiter = SlidingWindowRateLimiter::with_window(1, Duration::from_millis(50));
        limiter.try_acquire("key:127.0.0.1").await.expect("first");

        tokio::time::sleep(Duration::from_millis(60)).await;

        limiter
            .try_acquire("key:127.0.0.1")
            .await
            .expect("después de ventana");
    }
}
