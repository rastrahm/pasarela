//! Programa Anchor `payment-settlement` — liquidación on-chain del riel Solana (Fase 3).
//!
//! Ver [Casos-de-Uso UC-07](../../../Doc/Casos-de-Uso-ER-Flujos.md) y Plan §7.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub use state::{
    find_settlement_pda, PaymentProcessed, SettlementState, SETTLEMENT_SEED,
};

mod constraints;
mod state;

use constraints::{
    settlement_belongs_to_merchant, settlement_bump_matches, settlement_has_min_space,
};

declare_id!("4cKoeammHN8UjAbiJRw2DqxBPL1Mb1EaPQeJFFuo564B");

/// Programa de liquidación SPL para la pasarela multi-rail.
#[program]
pub mod payment_settlement {
    use super::*;

    /// @notice Bootstrap del programa — verificación de despliegue (Fase 3.1).
    /// @dev No modifica estado on-chain; reservado para smoke tests post-deploy.
    /// @param _ctx Contexto vacío (`Initialize` — sin cuentas requeridas).
    /// @return Result<()> Ok sin efectos de estado persistente.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("payment-settlement program initialized");
        Ok(())
    }

    /// @notice Crea e inicializa la PDA `SettlementState` de un comercio.
    /// @dev Seeds: `[SETTLEMENT_SEED, merchant.key()]`. Una PDA por merchant; contadores en cero.
    /// @param ctx Cuentas: `payer` (signer, rent), `merchant` (seed), `settlement_state` (init PDA), `system_program`.
    /// @return Result<()> Ok si la PDA queda inicializada con `total_amount = 0` y `payment_count = 0`.
    /// @return PaymentSettlementError::InvalidSettlementPda si la dirección derivada o bump no coinciden con seeds.
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
    /// @dev No usar en producción. Requiere firma del merchant dueño de la PDA (`has_one`).
    /// @param ctx Cuentas: `merchant` (signer), `settlement_state` (PDA mut).
    /// @param total_amount Valor a fijar en `SettlementState.total_amount` (base units SPL).
    /// @param payment_count Valor a fijar en `SettlementState.payment_count`.
    /// @return Result<()> Ok si la escritura es autorizada.
    /// @return PaymentSettlementError::Unauthorized si el signer no es el merchant de la PDA.
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

    /// @notice Crea PDA con espacio insuficiente — **solo tests** (TDD §7.4).
    /// @dev Fixture para verificar rechazo por `InsufficientAccountSpace` en `process_payment`.
    /// @param _ctx Cuentas: `payer` (signer), `merchant` (seed), `settlement_state` (16 bytes), `system_program`.
    /// @return Result<()> Ok si la cuenta queda creada con espacio menor que `SettlementState::LEN`.
    pub fn initialize_settlement_undersized(
        _ctx: Context<InitializeSettlementUndersized>,
    ) -> Result<()> {
        Ok(())
    }

    /// @notice Procesa un pago on-chain transferiendo tokens SPL al comercio.
    /// @dev Validaciones explícitas en `ProcessPayment` (Fase 3.6). Emite `PaymentProcessed` sin PII (UC-07 / D6).
    /// @param ctx Cuentas: `payer` (signer), `payer_token_account`, `merchant_token_account`, `merchant`, `settlement_state` (PDA), `token_program`.
    /// @param amount Cantidad de tokens SPL a transferir (base units del mint compartido).
    /// @param brand_code Código numérico de marca de tarjeta (sin PAN ni PII).
    /// @param settlement_rail_id Identificador del riel de liquidación (p. ej. `SolanaWallet = 3`).
    /// @return Result<()> Ok tras transferencia SPL, actualización de contadores PDA y emisión del evento.
    /// @return PaymentSettlementError::InvalidAmount si `amount == 0`.
    /// @return PaymentSettlementError::Unauthorized si owner de cuentas SPL o merchant de la PDA no coincide.
    /// @return PaymentSettlementError::InvalidMint si el mint del pagador difiere del comercio.
    /// @return PaymentSettlementError::InsufficientFunds si el saldo SPL del pagador es menor que `amount`.
    /// @return PaymentSettlementError::InsufficientAccountSpace si la PDA no alcanza `SettlementState::LEN`.
    /// @return PaymentSettlementError::InvalidSettlementPda si seeds o bump de la PDA son inválidos.
    /// @return PaymentSettlementError::AmountOverflow si `total_amount` o `payment_count` exceden `u64::MAX`.
    pub fn process_payment(
        ctx: Context<ProcessPayment>,
        amount: u64,
        brand_code: u8,
        settlement_rail_id: u64,
    ) -> Result<()> {
        let settlement_info = ctx.accounts.settlement_state.to_account_info();

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.merchant_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            amount,
        )?;

        {
            let mut data = settlement_info.try_borrow_mut_data()?;
            let mut state = SettlementState::try_deserialize(&mut &data[..])?;
            state.total_amount = state
                .total_amount
                .checked_add(amount)
                .ok_or(PaymentSettlementError::AmountOverflow)?;
            state.payment_count = state
                .payment_count
                .checked_add(1)
                .ok_or(PaymentSettlementError::AmountOverflow)?;
            state.try_serialize(&mut &mut data[..])?;
        }

        let clock = Clock::get()?;
        emit!(PaymentProcessed {
            amount,
            brand_code,
            settlement_rail_id,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}

/// Cuentas para bootstrap del programa (Fase 3.1).
///
/// Sin cuentas requeridas — instrucción no-op de verificación de despliegue.
#[derive(Accounts)]
pub struct Initialize {}

/// Cuentas para crear la PDA `SettlementState` (Fase 3.3).
///
/// | Cuenta | Rol |
/// |--------|-----|
/// | `payer` | Signer que paga rent-exempt de la PDA |
/// | `merchant` | Pubkey seed; no requiere firma en init |
/// | `settlement_state` | PDA init con seeds `[SETTLEMENT_SEED, merchant]` |
/// | `system_program` | System Program para `init` |
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

/// Fixture de test: PDA con espacio menor que `SettlementState::LEN` (TDD §7.4).
///
/// Usada exclusivamente por `initialize_settlement_undersized` para probar
/// rechazo por `InsufficientAccountSpace` en `process_payment`.
#[derive(Accounts)]
pub struct InitializeSettlementUndersized<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: merchant pubkey usada como seed.
    pub merchant: UncheckedAccount<'info>,
    /// CHECK: PDA de test con espacio 16 bytes (< SettlementState::LEN).
    #[account(
        init,
        payer = payer,
        space = 16,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump,
    )]
    pub settlement_state: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

/// Cuentas para ajuste de contadores en tests (Fase 3.2).
///
/// | Cuenta | Rol |
/// |--------|-----|
/// | `merchant` | Signer dueño de la PDA (`has_one`) |
/// | `settlement_state` | PDA mut con seeds `[SETTLEMENT_SEED, merchant]` |
#[derive(Accounts)]
pub struct SetSettlementTotals<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,
    #[account(
        mut,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump = settlement_state.bump,
        has_one = merchant @ PaymentSettlementError::Unauthorized,
    )]
    pub settlement_state: Account<'info, SettlementState>,
}

/// Cuentas para `process_payment` — constraints explícitos (Fase 3.6 / Arquitectura §7.2).
///
/// | Cuenta | Rol |
/// |--------|-----|
/// | `payer` | Signer autorizado; owner de `payer_token_account` |
/// | `payer_token_account` | Cuenta SPL origen (debita `amount`) |
/// | `merchant_token_account` | Cuenta SPL destino del comercio |
/// | `merchant` | Pubkey seed de la PDA; validado contra estado on-chain |
/// | `settlement_state` | PDA acumuladora con seeds `[SETTLEMENT_SEED, merchant]` |
/// | `token_program` | Programa SPL Token canonical |
#[derive(Accounts)]
#[instruction(amount: u64, brand_code: u8, settlement_rail_id: u64)]
pub struct ProcessPayment<'info> {
    /// Signer autorizado: debe ser owner de `payer_token_account`.
    #[account(
        mut,
        constraint = amount > 0 @ PaymentSettlementError::InvalidAmount,
    )]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = payer_token_account.owner == payer.key() @ PaymentSettlementError::Unauthorized,
        constraint = payer_token_account.mint == merchant_token_account.mint @ PaymentSettlementError::InvalidMint,
        constraint = payer_token_account.amount >= amount @ PaymentSettlementError::InsufficientFunds,
    )]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = merchant_token_account.owner == merchant.key() @ PaymentSettlementError::Unauthorized,
    )]
    pub merchant_token_account: Account<'info, TokenAccount>,
    /// CHECK: merchant pubkey — seed de la PDA; validado contra estado on-chain.
    pub merchant: UncheckedAccount<'info>,
    /// CHECK: PDA `[SETTLEMENT_SEED, merchant]`; espacio, seeds, bump y merchant explícitos.
    #[account(
        mut,
        constraint = settlement_has_min_space(settlement_state.as_ref()) @ PaymentSettlementError::InsufficientAccountSpace,
        constraint = settlement_belongs_to_merchant(settlement_state.as_ref(), &merchant.key()) @ PaymentSettlementError::Unauthorized,
        constraint = settlement_bump_matches(settlement_state.as_ref(), &merchant.key(), &crate::ID) @ PaymentSettlementError::InvalidSettlementPda,
        seeds = [SETTLEMENT_SEED, merchant.key().as_ref()],
        bump,
    )]
    pub settlement_state: UncheckedAccount<'info>,
    /// Programa SPL Token canonical (owner de las cuentas de tokens).
    pub token_program: Program<'info, Token>,
}

/// Errores del programa `payment-settlement` — referenciados en `@return` de cada instrucción.
#[error_code]
pub enum PaymentSettlementError {
    /// Signer no autorizado para la operación (owner mismatch o merchant PDA incorrecto).
    #[msg("Signer is not authorized for this payment")]
    Unauthorized,
    /// Dirección PDA, bump o seed de merchant inválidos.
    #[msg("Settlement PDA address, bump or merchant seed is invalid")]
    InvalidSettlementPda,
    /// Cuenta PDA con longitud de datos insuficiente para deserializar `SettlementState`.
    #[msg("Settlement account data length is insufficient")]
    InsufficientAccountSpace,
    /// Desbordamiento aritmético en `total_amount` o `payment_count`.
    #[msg("Arithmetic overflow in payment amount or counters")]
    AmountOverflow,
    /// Mint SPL del pagador distinto al del comercio.
    #[msg("Token mint mismatch between payer and merchant accounts")]
    InvalidMint,
    /// Monto de pago igual a cero.
    #[msg("Payment amount must be greater than zero")]
    InvalidAmount,
    /// Saldo SPL insuficiente en la cuenta del pagador.
    #[msg("Payer token account has insufficient balance for this payment")]
    InsufficientFunds,
}
