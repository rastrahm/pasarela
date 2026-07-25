use anchor_lang::prelude::*;

/// Seed de la PDA `SettlementState` — ver Arquitectura §7.2 / Casos-de-Uso §3.4.
pub const SETTLEMENT_SEED: &[u8] = b"settlement";

/// Estado acumulado de liquidaciones por comercio (PDA).
///
/// Seeds: `[SETTLEMENT_SEED, merchant.as_ref()]`
/// Program owner: `payment-settlement`
#[account]
#[derive(InitSpace)]
pub struct SettlementState {
    /// Comercio dueño de esta PDA (repetido on-chain para `has_one` en instrucciones).
    pub merchant: Pubkey,
    /// Suma acumulada de montos liquidados (tokens SPL base units).
    pub total_amount: u64,
    /// Cantidad de pagos procesados exitosamente.
    pub payment_count: u64,
    /// Bump canonical de la PDA — almacenado para evitar recalcular en CPI.
    pub bump: u8,
}

impl SettlementState {
    /// Espacio rent-exempt incluyendo discriminador Anchor (8 bytes).
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}

/// Deriva la PDA `SettlementState` para un comercio.
///
/// Seeds: `[SETTLEMENT_SEED, merchant.as_ref()]`.
///
/// @param program_id Program ID del contrato Anchor.
/// @param merchant Pubkey del comercio (seed).
/// @return (Pubkey, u8) Dirección PDA y bump canonical.
pub fn find_settlement_pda(program_id: &Pubkey, merchant: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SETTLEMENT_SEED, merchant.as_ref()], program_id)
}

/// Evento de auditoría on-chain — sin PII (Arquitectura §7.3 / UC-07).
///
/// Campos permitidos: monto, código de marca numérico, riel y timestamp.
/// Prohibido: PAN, nombre, email u otro dato personal identificable.
#[event]
pub struct PaymentProcessed {
    pub amount: u64,
    pub brand_code: u8,
    pub settlement_rail_id: u64,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_state_len_matches_init_space() {
        assert_eq!(SettlementState::LEN, 8 + SettlementState::INIT_SPACE);
    }
}
