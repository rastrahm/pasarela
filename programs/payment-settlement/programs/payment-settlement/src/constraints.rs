//! Helpers de validación para constraints explícitos (Fase 3.6).

use anchor_lang::prelude::*;

use crate::state::{SettlementState, SETTLEMENT_SEED};

/// Verifica que la cuenta PDA tenga espacio rent-exempt suficiente para `SettlementState`.
pub fn settlement_has_min_space(settlement: &AccountInfo) -> bool {
    settlement.data_len() >= SettlementState::LEN
}

/// Verifica que el merchant embebido en la PDA coincida con la clave seed pasada.
pub fn settlement_belongs_to_merchant(settlement: &AccountInfo, merchant: &Pubkey) -> bool {
    if !settlement_has_min_space(settlement) {
        return true;
    }
    let Ok(data) = settlement.try_borrow_data() else {
        return false;
    };
    let Ok(state) = SettlementState::try_deserialize(&mut &data[..]) else {
        return false;
    };
    state.merchant == *merchant
}

/// Verifica que el bump almacenado en la PDA sea el canonical de las seeds.
pub fn settlement_bump_matches(settlement: &AccountInfo, merchant: &Pubkey, program_id: &Pubkey) -> bool {
    if !settlement_has_min_space(settlement) {
        return true;
    }
    let Ok(data) = settlement.try_borrow_data() else {
        return false;
    };
    let Ok(state) = SettlementState::try_deserialize(&mut &data[..]) else {
        return false;
    };
    let (_, canonical_bump) = Pubkey::find_program_address(
        &[SETTLEMENT_SEED, merchant.as_ref()],
        program_id,
    );
    state.bump == canonical_bump
}
