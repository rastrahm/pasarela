/**
 * Helpers de interacción RTL para flujos de checkout en tests (paso 5.10).
 */
import { screen } from '@testing-library/react'
import type { UserEvent } from '@testing-library/user-event'
import { vi } from 'vitest'

import type { FundingType } from '../schemas/funding'

/**
 * Rellena y valida la tarjeta demo en CheckoutPage.
 *
 * @param user - Instancia de `userEvent.setup()`.
 */
export async function validateDemoCard(user: UserEvent): Promise<void> {
  await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
  await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
}

/**
 * Selecciona un riel de liquidación por etiqueta accesible del radiogroup.
 *
 * @param user - Instancia de `userEvent.setup()`.
 * @param railLabel - Regex del label visible del riel.
 */
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

/**
 * Selecciona riel por valor `FundingType` (snake_case).
 *
 * @param user - Instancia de `userEvent.setup()`.
 * @param fundingType - Riel a seleccionar.
 */
export async function selectFundingType(
  user: UserEvent,
  fundingType: FundingType,
): Promise<void> {
  await selectFundingRail(user, RAIL_LABELS[fundingType])
}

/**
 * Pulsa «Confirmar pago» en CheckoutPage.
 *
 * @param user - Instancia de `userEvent.setup()`.
 */
export async function submitCheckoutPayment(user: UserEvent): Promise<void> {
  await user.click(screen.getByRole('button', { name: /confirmar pago/i }))
}

/**
 * Flujo de interacción: validar tarjeta demo → elegir riel → confirmar pago.
 *
 * @param user - Instancia de `userEvent.setup()`.
 * @param fundingType - Riel enviado en el checkout.
 */
export async function checkoutWithRail(
  user: UserEvent,
  fundingType: FundingType,
): Promise<void> {
  await validateDemoCard(user)
  await selectFundingType(user, fundingType)
  await submitCheckoutPayment(user)
}

/**
 * Devuelve el body JSON del último `fetch` mockeado en el test.
 *
 * @returns Objeto parseado del cuerpo POST.
 */
export function getLastFetchJsonBody(): Record<string, unknown> {
  const calls = vi.mocked(fetch).mock.calls
  const init = calls[calls.length - 1]?.[1]
  return JSON.parse(String(init?.body)) as Record<string, unknown>
}

/**
 * Devuelve los headers del último `fetch` mockeado en el test.
 *
 * @returns Mapa de headers HTTP enviados.
 */
export function getLastFetchHeaders(): Record<string, string> {
  const calls = vi.mocked(fetch).mock.calls
  const init = calls[calls.length - 1]?.[1]
  return (init?.headers ?? {}) as Record<string, string>
}

/**
 * Construye respuesta mock de checkout settled para un riel dado.
 *
 * @param fundingType - Riel usado en la respuesta.
 * @param proof - Comprobante de liquidación (ACH, CEX, SOL, etc.).
 * @returns Objeto compatible con {@link CheckoutResponse}.
 */
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
