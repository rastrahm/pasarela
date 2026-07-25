//! Programa Anchor `payment-settlement` — liquidación on-chain del riel Solana (Fase 3).
//!
//! Ver [Casos-de-Uso UC-07](../../../Doc/Casos-de-Uso-ER-Flujos.md) y Plan §7.

use anchor_lang::prelude::*;

pub use state::{
    find_settlement_pda, PaymentProcessed, SettlementState, SETTLEMENT_SEED,
};

mod state;

declare_id!("4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B");

/// Programa de liquidación SPL para la pasarela multi-rail.
#[program]
pub mod payment_settlement {
    use super::*;

    /// @notice Inicializa el programa (bootstrap Fase 3.1).
    /// @param _ctx Contexto vacío de bootstrap.
    /// @return Result<()> Ok si la instrucción se ejecuta sin error.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("payment-settlement program initialized");
        Ok(())
    }

    /// @notice Crea e inicializa la PDA `SettlementState` de un comercio.
    /// @dev Seeds: `[SETTLEMENT_SEED, merchant.key()]`. Una PDA por merchant.
    /// @param ctx Pagador (rent), merchant (seed) y system program.
    /// @return Result<()> Ok si la PDA queda inicializada con contadores en cero.
    pub fn initialize_settlement(ctx: Context<InitializeSettlement>) -> Result<()> {
        let merchant_key = ctx.accounts.merchant.key();
        let (expected_pda, expected_bump) = find_settlement_pda(ctx.program_id, &merchant_key);
        require_keys_eq!(
            ctx.accounts.settlement_state.key(),
            expected_pda,
            PaymentSettlementError::InvalidSettlementPda
        );
        require_eq!(
            ctx.bumps.settlement_state,
            expected_bump,
            PaymentSettlementError::InvalidSettlementPda
        );

        let state = &mut ctx.accounts.settlement_state;
        state.merchant = merchant_key;
        state.total_amount = 0;
        state.payment_count = 0;
        state.bump = ctx.bumps.settlement_state;
        Ok(())
    }

    /// @notice Ajusta contadores de la PDA — **solo tests locales** (setup TDD 3.2).
    /// @param ctx Merchant firmante y PDA asociada.
    /// @param total_amount Valor acumulado a fijar.
    /// @param payment_count Contador de pagos a fijar.
    /// @return Result<()> Ok si la escritura es autorizada.
    pub fn set_settlement_totals(
        ctx: Context<SetSettlementTotals>,
        total_amount: u64,
        payment_count: u64,
    ) -> Result<()> {
        let state = &mut ctx.accounts.settlement_state;
        state.total_amount = total_amount;
        state.payment_count = payment_count;
        Ok(())
    }

    /// @notice Procesa un pago on-chain transferiendo tokens SPL al comercio.
    /// @dev Lógica de liquidación: Fase 3.4. PDA validada por seeds (Fase 3.3).
    /// @param _ctx Cuentas SPL, PDA `SettlementState` y programas del sistema.
    /// @param _amount Cantidad de tokens SPL a transferir.
    /// @param _brand_code Código numérico de marca (sin PAN).
    /// @param _settlement_rail_id Identificador del riel de liquidación.
    /// @return Result<()> Ok tras transferencia y actualización de contadores.
    pub fn process_payment(
        _ctx: Context<ProcessPayment>,
        _amount: u64,
        _brand_code: u8,
        _settlement_rail_id: u64,
    ) -> Result<()> {
        err!(PaymentSettlementError::NotImplemented)
    }
}

/// Cuentas para bootstrap del programa (Fase 3.1).
#[derive(Accounts)]
pub struct Initialize {}

/// Cuentas para crear la PDA `SettlementState` (Fase 3.3).
#[derive(Accounts)]
#[instruction()]
pub struct InitializeSettlement<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: merchant pubkey usada como seed; no requiere firma en el init.
    pub merchant: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = SettlementState::LEN,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump,
    )]
    pub settlement_state: Account<'info, SettlementState>,
    pub system_program: Program<'info, System>,
}

/// Cuentas para ajuste de contadores en tests (Fase 3.2).
#[derive(Accounts)]
pub struct SetSettlementTotals<'info> {
    pub merchant: Signer<'info>,
    #[account(
        mut,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump = settlement_state.bump,
        has_one = merchant @ PaymentSettlementError::Unauthorized,
    )]
    pub settlement_state: Account<'info, SettlementState>,
}

/// Cuentas para `process_payment`.
#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    pub payer: Signer<'info>,
    #[account(mut)]
    pub payer_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    #[account(mut)]
    pub merchant_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    /// CHECK: merchant pubkey — seed de la PDA `SettlementState`.
    pub merchant: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump = settlement_state.bump,
        has_one = merchant @ PaymentSettlementError::Unauthorized,
        constraint = settlement_state.to_account_info().data_len() >= SettlementState::LEN @ PaymentSettlementError::InsufficientAccountSpace,
    )]
    pub settlement_state: Account<'info, SettlementState>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum PaymentSettlementError {
    #[msg("Instruction not implemented yet (Fase 3.4)")]
    NotImplemented,
    #[msg("Signer is not authorized for this payment")]
    Unauthorized,
    #[msg("Settlement PDA address or bump does not match merchant seeds")]
    InvalidSettlementPda,
    #[msg("Settlement account data length is insufficient")]
    InsufficientAccountSpace,
    #[msg("Arithmetic overflow in payment amount or counters")]
    AmountOverflow,
    #[msg("Token mint mismatch between payer and merchant accounts")]
    InvalidMint,
}
