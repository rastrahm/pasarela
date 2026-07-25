//! Programa Anchor `payment-settlement` — liquidación on-chain del riel Solana (Fase 3).
//!
//! Instrucciones previstas:
//! - `process_payment`: transferencia SPL + actualización de PDA `SettlementState`
//! - Evento `PaymentProcessed` sin PII (solo amount, brand_code, settlement_rail_id)
//!
//! Ver [Casos-de-Uso UC-07](../../../Doc/Casos-de-Uso-ER-Flujos.md) y Plan §7.

use anchor_lang::prelude::*;

declare_id!("4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B");

/// Programa de liquidación SPL para la pasarela multi-rail.
#[program]
pub mod payment_settlement {
    use super::*;

    /// @notice Inicializa el programa (placeholder Fase 3.1 — reemplazado por `process_payment` en 3.4).
    /// @dev Verifica despliegue y genera IDL; sin estado persistente aún.
    /// @param _ctx Contexto vacío de bootstrap.
    /// @return Result<()> Ok si la instrucción se ejecuta sin error.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("payment-settlement program initialized");
        Ok(())
    }
}

/// Cuentas para bootstrap del programa (temporal — Fase 3.1).
#[derive(Accounts)]
pub struct Initialize {}
