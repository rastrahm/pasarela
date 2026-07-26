/**
 * E2E checkout — flujo UI completo por riel (Fase 6.1).
 *
 * Requiere stack real: Gateway + Oracle (+ simuladores según riel).
 * Si el Gateway no responde, los tests se omiten con mensaje explícito.
 */
import { test, expect } from '@playwright/test'

import {
  checkoutWithRail,
  isGatewayHealthy,
  openCheckout,
  settlementProofLabel,
  type FundingType,
} from '../helpers/checkout'

const RAILS: Array<{
  fundingType: FundingType
  railLabel: RegExp
}> = [
  {
    fundingType: 'traditional_bank',
    railLabel: /banco tradicional/i,
  },
  {
    fundingType: 'binance_cex',
    railLabel: /binance cex/i,
  },
  {
    fundingType: 'solana_wallet',
    railLabel: /solana wallet/i,
  },
]

test.describe('Checkout E2E — stack real', () => {
  test.beforeAll(async () => {
    const healthy = await isGatewayHealthy()
    test.skip(
      !healthy,
      `Gateway no disponible en ${process.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080'}. ` +
        'Levantá Oracle + Gateway antes de ejecutar E2E (ver tests/e2e/README.md).',
    )
  })

  test('muestra la página de checkout', async ({ page }) => {
    await openCheckout(page)
    await expect(page.getByRole('heading', { name: 'Checkout' })).toBeVisible()
    await expect(
      page.getByRole('button', { name: /usar datos de prueba/i }),
    ).toBeVisible()
  })

  for (const { fundingType, railLabel } of RAILS) {
    test(`checkout exitoso — riel ${fundingType}`, async ({ page }) => {
      await openCheckout(page)
      await checkoutWithRail(page, fundingType)

      await expect(page.getByText('Comprobante')).toBeVisible()
      await expect(page.getByText('Liquidada', { exact: true })).toBeVisible()
      await expect(page.getByText(railLabel)).toBeVisible()
      await expect(
        page.getByText(settlementProofLabel(fundingType)),
      ).toBeVisible()

      const proofValue = page.locator('dd.break-all.font-mono').first()
      await expect(proofValue).not.toBeEmpty()
    })
  }
})

test.describe('Checkout E2E — validación UI (sin backend)', () => {
  test('bloquea pago sin tarjeta validada', async ({ page }) => {
    await openCheckout(page)

    const payButton = page.getByRole('button', { name: /confirmar pago/i })
    await expect(payButton).toBeDisabled()
    await expect(page.getByText(/validá la tarjeta para habilitar/i)).toBeVisible()
  })

  test('permite validar tarjeta demo y habilita pago', async ({ page }) => {
    await openCheckout(page)
    await page.getByRole('button', { name: /usar datos de prueba/i }).click()
    await page.getByRole('button', { name: /validar tarjeta/i }).click()

    await expect(
      page.getByRole('button', { name: /confirmar pago/i }),
    ).toBeEnabled()
    await expect(page.getByText(/terminada en 1111/i)).toBeVisible()
  })
})
