import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

import { CheckoutPage } from './CheckoutPage'
import {
  checkoutWithRail,
  getLastFetchHeaders,
  getLastFetchJsonBody,
  mockSettledCheckoutResponse,
  selectFundingType,
  submitCheckoutPayment,
  validateDemoCard,
} from '../test/checkout-flow'
import { renderUi, screen, userEvent } from '../test/test-utils'

describe('CheckoutPage — interacción', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('envía payload completo al confirmar pago', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify(
          mockSettledCheckoutResponse('traditional_bank', 'ACH-MEM-001'),
        ),
        { status: 200 },
      ),
    )

    renderUi(<CheckoutPage />)

    await validateDemoCard(user)
    await selectFundingType(user, 'traditional_bank')
    await user.clear(screen.getByLabelText(/monto/i))
    await user.type(screen.getByLabelText(/monto/i), '250.5')
    await submitCheckoutPayment(user)

    expect(fetch).toHaveBeenCalledOnce()

    const body = getLastFetchJsonBody()
    expect(body.amount).toBe(250.5)
    expect(body.currency).toBe('USD')
    expect(body.funding_type).toBe('traditional_bank')
    expect(body.card).toMatchObject({
      pan: '4111111111111111',
      expiry_month: '12',
      expiry_year: '2030',
      cvv: '123',
      cardholder: 'Demo User',
    })

    const headers = getLastFetchHeaders()
    expect(headers.Authorization).toMatch(/^Bearer sk_/)
    expect(headers['Idempotency-Key']).toBeTruthy()
    expect(headers['Content-Type']).toBe('application/json')
  })

  it.each([
    ['traditional_bank', 'ACH-MEM-001', /referencia bancaria/i],
    ['binance_cex', 'CEX-MEM-ORD-99', /order id/i],
    ['solana_wallet', 'SOL-MEM-devnet-sig', /tx signature/i],
  ] as const)(
    'cambio de riel %s refleja proof distinto en la UI',
    async (fundingType, proof, proofLabel) => {
      const user = userEvent.setup()
      vi.mocked(fetch).mockResolvedValue(
        new Response(
          JSON.stringify(mockSettledCheckoutResponse(fundingType, proof)),
          { status: 200 },
        ),
      )

      renderUi(<CheckoutPage />)

      await checkoutWithRail(user, fundingType)

      const body = getLastFetchJsonBody()
      expect(body.funding_type).toBe(fundingType)

      expect(await screen.findByText('Liquidación completada')).toBeInTheDocument()
      expect(screen.getByText(proof)).toBeInTheDocument()
      expect(screen.getByText(proofLabel)).toBeInTheDocument()
    },
  )

  it('actualiza funding_type si el usuario cambia de riel antes del submit', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify(
          mockSettledCheckoutResponse('solana_wallet', 'SOL-MEM-xyz'),
        ),
        { status: 200 },
      ),
    )

    renderUi(<CheckoutPage />)

    await validateDemoCard(user)
    await selectFundingType(user, 'traditional_bank')
    await selectFundingType(user, 'solana_wallet')
    await submitCheckoutPayment(user)

    expect(getLastFetchJsonBody().funding_type).toBe('solana_wallet')
  })

  it('muestra error local con monto inválido sin llamar al Gateway', async () => {
    const user = userEvent.setup()

    renderUi(<CheckoutPage />)

    await validateDemoCard(user)
    await user.clear(screen.getByLabelText(/monto/i))
    await user.type(screen.getByLabelText(/monto/i), '0')
    await submitCheckoutPayment(user)

    expect(fetch).not.toHaveBeenCalled()
    expect(await screen.findByRole('alert')).toHaveTextContent(/monto válido/i)
  })

  it('bloquea submit si la tarjeta no pasa validación Zod', async () => {
    const user = userEvent.setup()

    renderUi(<CheckoutPage />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.clear(screen.getByLabelText(/número de tarjeta/i))
    await user.type(screen.getByLabelText(/número de tarjeta/i), '4111111111111112')
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))

    expect(screen.getByText(/luhn/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /confirmar pago/i })).toBeDisabled()
    expect(fetch).not.toHaveBeenCalled()
  })

  it('muestra errores de campo al validar tarjeta vacía', async () => {
    const user = userEvent.setup()

    renderUi(<CheckoutPage />)

    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))

    expect(screen.getByText(/revisá los datos/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /confirmar pago/i })).toBeDisabled()
  })

  it('registra el fallo en TransactionViewer tras error del Gateway', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({
          error_code: 'RAIL_UNAVAILABLE',
          message: 'riel no disponible',
        }),
        { status: 503 },
      ),
    )

    renderUi(<CheckoutPage />)

    await checkoutWithRail(user, 'binance_cex')

    expect(await screen.findByText(/checkout fallido/i)).toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent(/riel no disponible/i)
  })
})
