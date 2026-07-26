//! Estado mutable del simulador (saldos Spot en memoria).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rust_decimal::Decimal;

use crate::config::AppConfig;

/// Estado compartido del servicio — saldos Spot mutables entre consultas y débitos.
#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    balances: RwLock<HashMap<String, Decimal>>,
}

impl AppState {
    /// Inicializa saldos desde la configuración estática.
    pub fn new(config: AppConfig) -> Arc<Self> {
        let mut balances = config.balances_by_currency.clone();
        if balances.is_empty() {
            for currency in ["USD", "USDC", "USDT"] {
                balances.insert(currency.to_string(), config.default_balance);
            }
        }

        Arc::new(Self {
            config,
            balances: RwLock::new(balances),
        })
    }

    /// Saldo Spot bruto disponible para una moneda.
    pub fn balance_for(&self, currency: &str) -> Decimal {
        let normalized = currency.to_ascii_uppercase();
        self.balances
            .read()
            .map(|map| map.get(&normalized).copied().unwrap_or(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO)
    }

    /// Aplica débito atómico si hay saldo suficiente tras validar spread buffer.
    ///
    /// # Returns
    /// Nuevo saldo restante o `None` si fondos insuficientes.
    pub fn try_debit(
        &self,
        currency: &str,
        amount: Decimal,
        spread_buffer_pct: Decimal,
    ) -> Option<Decimal> {
        if amount <= Decimal::ZERO {
            return None;
        }

        let normalized = currency.to_ascii_uppercase();
        let mut balances = self.balances.write().ok()?;
        let balance = balances.get(&normalized).copied().unwrap_or(Decimal::ZERO);

        let factor = Decimal::ONE - spread_buffer_pct;
        let effective = balance * factor;

        if balance < amount || effective < amount {
            return None;
        }

        let remaining = balance - amount;
        balances.insert(normalized, remaining);
        Some(remaining)
    }
}
