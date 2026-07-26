//! Almacén in-memory del Gateway (tests y arranque sin DATABASE_URL).

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::RwLock;

use domain::MerchantId;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::routes::CheckoutResponse;
use crate::services::idempotency::CachedCheckoutResult;

use super::error::PersistenceError;
use super::models::{AuditEvent, SettlementRecord, TransactionRecord};

#[derive(Debug, Clone, PartialEq)]
enum IdempotencyEntry {
    InFlight { fingerprint: String },
    Completed {
        fingerprint: String,
        result: CachedCheckoutResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IdempotencyCompositeKey {
    merchant_id: Uuid,
    key: String,
}

/// Store in-memory para transacciones, settlements, auditoría e idempotencia.
#[derive(Default)]
pub struct InMemoryStore {
    transactions: RwLock<HashMap<Uuid, TransactionRecord>>,
    settlements: RwLock<HashMap<Uuid, SettlementRecord>>,
    audit_log: RwLock<Vec<AuditEvent>>,
    idempotency: RwLock<HashMap<IdempotencyCompositeKey, IdempotencyEntry>>,
}

impl InMemoryStore {
    pub fn upsert_transaction(&self, record: &TransactionRecord) -> Result<(), PersistenceError> {
        let mut map = self
            .transactions
            .write()
            .map_err(|_| PersistenceError::LockPoisoned)?;
        map.insert(record.transaction_id, record.clone());
        Ok(())
    }

    pub fn get_transaction(
        &self,
        id: Uuid,
        merchant_id: MerchantId,
    ) -> Result<Option<TransactionRecord>, PersistenceError> {
        let map = self
            .transactions
            .read()
            .map_err(|_| PersistenceError::LockPoisoned)?;
        Ok(map
            .get(&id)
            .filter(|record| record.merchant_id == merchant_id)
            .cloned())
    }

    pub fn insert_settlement(&self, record: &SettlementRecord) -> Result<(), PersistenceError> {
        let mut map = self
            .settlements
            .write()
            .map_err(|_| PersistenceError::LockPoisoned)?;
        map.insert(record.id, record.clone());
        Ok(())
    }

    pub fn append_audit(&self, event: &AuditEvent) -> Result<(), PersistenceError> {
        let mut log = self
            .audit_log
            .write()
            .map_err(|_| PersistenceError::LockPoisoned)?;
        log.push(event.clone());
        Ok(())
    }

    pub fn idempotency_begin(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
    ) -> Result<Option<CachedCheckoutResult>, PersistenceError> {
        let composite = IdempotencyCompositeKey {
            merchant_id: merchant_id.0,
            key: key.to_string(),
        };

        let mut map = self
            .idempotency
            .write()
            .map_err(|_| PersistenceError::LockPoisoned)?;

        match map.entry(composite) {
            Entry::Vacant(slot) => {
                slot.insert(IdempotencyEntry::InFlight {
                    fingerprint: fingerprint.to_string(),
                });
                Ok(None)
            }
            Entry::Occupied(slot) => match slot.get() {
                IdempotencyEntry::InFlight { fingerprint: existing } if existing == fingerprint => {
                    Err(PersistenceError::Internal(
                        "checkout idempotente en curso".to_string(),
                    ))
                }
                IdempotencyEntry::InFlight { .. } => Err(PersistenceError::Internal(
                    "Idempotency-Key en uso con otra solicitud".to_string(),
                )),
                IdempotencyEntry::Completed {
                    fingerprint: existing,
                    result,
                } if existing == fingerprint => Ok(Some(result.clone())),
                IdempotencyEntry::Completed { .. } => Err(PersistenceError::Internal(
                    "Idempotency-Key reutilizada con payload distinto".to_string(),
                )),
            },
        }
    }

    pub fn idempotency_complete(
        &self,
        merchant_id: MerchantId,
        key: &str,
        fingerprint: &str,
        result: &Result<CheckoutResponse, GatewayError>,
    ) -> Result<(), PersistenceError> {
        let composite = IdempotencyCompositeKey {
            merchant_id: merchant_id.0,
            key: key.to_string(),
        };

        let cached = match result {
            Ok(response) => CachedCheckoutResult::Success(response.clone()),
            Err(error) => CachedCheckoutResult::Error(error.clone()),
        };

        let mut map = self
            .idempotency
            .write()
            .map_err(|_| PersistenceError::LockPoisoned)?;
        map.insert(
            composite,
            IdempotencyEntry::Completed {
                fingerprint: fingerprint.to_string(),
                result: cached,
            },
        );
        Ok(())
    }
}
