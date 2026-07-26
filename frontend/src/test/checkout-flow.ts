import { screen } from '@testing-library/react'
import type { UserEvent } from '@testing-library/user-event'
import { vi } from 'vitest'

import type { FundingType } from '../schemas/funding'

/** Rellena y valida la tarjeta demo en el checkout. */
export async function validateDemoCard(user: UserEvent): Promise<void> {
  await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
  await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
}

/** Selecciona un riel de liquidación por etiqueta accesible. */
export async function selectFundingRail(
  user: UserEvent,
  railLabel: RegExp,
): Promise<void> {
  await user.click(screen.getByRole('radio', { name: railLabel }))
}

const RAIL_LABELS: Record<FundingType, RegExp> = {
  traditional_bank: /banco tradicional/i,
  binance_cex: /binance cex/i,
  solana_wallet: /solana wallet/i,
}

/** Selecciona riel por valor snake_case. */
export async function selectFundingType(
  user: UserEvent,
  fundingType: FundingType,
): Promise<void> {
  await selectFundingRail(user, RAIL_LABELS[fundingType])
}

/** Confirma el pago en CheckoutPage. */
export async function submitCheckoutPayment(user: UserEvent): Promise<void> {
  await user.click(screen.getByRole('button', { name: /confirmar pago/i }))
}

/** Flujo completo hasta submit. */
export async function checkoutWithRail(
  user: UserEvent,
  fundingType: FundingType,
): Promise<void> {
  await validateDemoCard(user)
  await selectFundingType(user, fundingType)
  await submitCheckoutPayment(user)
}

/** Devuelve el body JSON del último fetch mockeado. */
export function getLastFetchJsonBody(): Record<string, unknown> {
  const calls = vi.mocked(fetch).mock.calls
  const init = calls[calls.length - 1]?.[1]
  return JSON.parse(String(init?.body)) as Record<string, unknown>
}

/** Devuelve headers del último fetch mockeado. */
export function getLastFetchHeaders(): Record<string, string> {
  const calls = vi.mocked(fetch).mock.calls
  const init = calls[calls.length - 1]?.[1]
  return (init?.headers ?? {}) as Record<string, string>
}

/** Mock de checkout exitoso por riel (proof distinto por riel — gate Fase 5). */
export function mockSettledCheckoutResponse(
  fundingType: FundingType,
  proof: string,
) {
  return {
    transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    status: 'settled',
    rail_used: fundingType,
    settlement_proof: proof,
  }
}
