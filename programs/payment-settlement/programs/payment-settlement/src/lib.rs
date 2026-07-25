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

    /// @notice Crea PDA con espacio insuficiente — **solo tests** (TDD §7.4).
    /// @param _ctx Pagador y merchant (seed).
    /// @return Result<()> Ok si la cuenta queda creada con espacio reducido.
    pub fn initialize_settlement_undersized(
        _ctx: Context<InitializeSettlementUndersized>,
    ) -> Result<()> {
        Ok(())
    }

    /// @notice Procesa un pago on-chain transferiendo tokens SPL al comercio.
    /// @dev Validaciones de cuentas en `ProcessPayment` (Fase 3.6). Emite `PaymentProcessed` sin PII.
    /// @param ctx Cuentas SPL, PDA `SettlementState` y token program.
    /// @param amount Cantidad de tokens SPL a transferir (base units).
    /// @param brand_code Código numérico de marca (sin PAN).
    /// @param settlement_rail_id Identificador del riel de liquidación.
    /// @return Result<()> Ok tras transferencia, contadores y evento emitidos.
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

/// Fixture de test: PDA con espacio menor que `SettlementState::LEN`.
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

#[error_code]
pub enum PaymentSettlementError {
    #[msg("Signer is not authorized for this payment")]
    Unauthorized,
    #[msg("Settlement PDA address, bump or merchant seed is invalid")]
    InvalidSettlementPda,
    #[msg("Settlement account data length is insufficient")]
    InsufficientAccountSpace,
    #[msg("Arithmetic overflow in payment amount or counters")]
    AmountOverflow,
    #[msg("Token mint mismatch between payer and merchant accounts")]
    InvalidMint,
    #[msg("Payment amount must be greater than zero")]
    InvalidAmount,
    #[msg("Payer token account has insufficient balance for this payment")]
    InsufficientFunds,
}
