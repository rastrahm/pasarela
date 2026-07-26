/**
 * Helpers de interacción Playwright — espejo de `frontend/src/test/checkout-flow.ts`.
 */
import type { Page } from '@playwright/test'

export type FundingType =
  | 'traditional_bank'
  | 'binance_cex'
  | 'solana_wallet'

const RAIL_LABELS: Record<FundingType, RegExp> = {
  traditional_bank: /banco tradicional/i,
  binance_cex: /binance cex/i,
  solana_wallet: /solana wallet/i,
}

const PROOF_LABELS: Record<FundingType, RegExp> = {
  traditional_bank: /referencia bancaria/i,
  binance_cex: /order id/i,
  solana_wallet: /tx signature/i,
}

/** URL del Gateway para healthcheck previo a E2E real. */
export function gatewayBaseUrl(): string {
  return process.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080'
}

/** Comprueba que el Gateway responde en `/health`. */
export async function isGatewayHealthy(): Promise<boolean> {
  try {
    const response = await fetch(`${gatewayBaseUrl()}/health`)
    return response.ok
  } catch {
    return false
  }
}

/** Navega al checkout y espera el título principal. */
export async function openCheckout(page: Page): Promise<void> {
  await page.goto('/')
  await page.getByRole('heading', { name: 'Checkout' }).waitFor()
}

/** Rellena y valida la tarjeta demo. */
export async function validateDemoCard(page: Page): Promise<void> {
  await page.getByRole('button', { name: /usar datos de prueba/i }).click()
  await page.getByRole('button', { name: /validar tarjeta/i }).click()
  await page.getByText(/tarjeta lista · terminada en/i).waitFor()
}

/** Selecciona un riel de liquidación. */
export async function selectFundingType(
  page: Page,
  fundingType: FundingType,
): Promise<void> {
  await page.getByRole('radio', { name: RAIL_LABELS[fundingType] }).click()
}

/** Confirma el pago y espera resultado (comprobante o error). */
export async function submitCheckoutPayment(page: Page): Promise<void> {
  await page.getByRole('button', { name: /confirmar pago/i }).click()
  await Promise.race([
    page.getByText('Comprobante').waitFor({ state: 'visible' }),
    page.getByRole('alert').waitFor({ state: 'visible' }),
  ])
}

/** Flujo completo: tarjeta demo → riel → confirmar. */
export async function checkoutWithRail(
  page: Page,
  fundingType: FundingType,
): Promise<void> {
  await validateDemoCard(page)
  await selectFundingType(page, fundingType)
  await submitCheckoutPayment(page)
}

/** Etiqueta del comprobante según riel (para aserciones). */
export function settlementProofLabel(fundingType: FundingType): RegExp {
  return PROOF_LABELS[fundingType]
}
