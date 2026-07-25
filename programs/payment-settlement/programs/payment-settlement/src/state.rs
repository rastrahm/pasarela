use anchor_lang::prelude::*;

/// Seed de la PDA `SettlementState` — ver Arquitectura §7.2.
pub const SETTLEMENT_SEED: &[u8] = b"settlement";

/// Estado acumulado de liquidaciones por comercio (PDA).
#[account]
pub struct SettlementState {
    pub merchant: Pubkey,
    pub total_amount: u64,
    pub payment_count: u64,
    pub bump: u8,
}

impl SettlementState {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

/// Evento de auditoría on-chain — sin PII (Arquitectura §7.3).
#[event]
pub struct PaymentProcessed {
    pub amount: u64,
    pub brand_code: u8,
    pub settlement_rail_id: u64,
    pub timestamp: i64,
}
