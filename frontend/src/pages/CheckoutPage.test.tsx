import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

import { CheckoutPage } from './CheckoutPage'
import { renderUi, screen, userEvent } from '../test/test-utils'

const SUCCESS_BODY = {
  transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  status: 'settled',
  rail_used: 'binance_cex',
  settlement_proof: 'CEX-MEM-001',
}

describe('CheckoutPage', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('deshabilita confirmar pago sin tarjeta validada', () => {
    renderUi(<CheckoutPage />)

    expect(screen.getByRole('button', { name: /confirmar pago/i })).toBeDisabled()
  })

  it('ejecuta checkout contra el Gateway tras validar tarjeta', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(JSON.stringify(SUCCESS_BODY), { status: 200 }),
    )

    renderUi(<CheckoutPage />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
    await user.click(screen.getByRole('radio', { name: /binance cex/i }))
    await user.click(screen.getByRole('button', { name: /confirmar pago/i }))

    expect(fetch).toHaveBeenCalledOnce()
    expect(await screen.findByText('Liquidación completada')).toBeInTheDocument()
    expect(screen.getByText('CEX-MEM-001')).toBeInTheDocument()
    expect(screen.getByText('Comprobante')).toBeInTheDocument()
  })

  it('muestra error UX ante respuesta 503', async () => {
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

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
    await user.click(screen.getByRole('button', { name: /confirmar pago/i }))

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(/riel no disponible/i)
    expect(alert).toHaveTextContent(/otro riel/i)
    expect(alert).toHaveTextContent(/HTTP 503/)
  })

  it('muestra error UX ante respuesta 402', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({
          error_code: 'INSUFFICIENT_FUNDS',
          message: 'fondos insuficientes',
        }),
        { status: 402 },
      ),
    )

    renderUi(<CheckoutPage />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
    await user.click(screen.getByRole('button', { name: /confirmar pago/i }))

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(/fondos insuficientes/i)
    expect(alert).toHaveTextContent(/HTTP 402/)
  })

  it('muestra error UX ante respuesta 422 INVALID_CARD', async () => {
    const user = userEvent.setup()
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({
          error_code: 'INVALID_CARD',
          message: 'tarjeta inválida',
        }),
        { status: 422 },
      ),
    )

    renderUi(<CheckoutPage />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))
    await user.click(screen.getByRole('button', { name: /confirmar pago/i }))

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(/tarjeta inválida/i)
    expect(alert).toHaveTextContent(/datos de prueba/i)
    expect(alert).toHaveTextContent(/HTTP 422/)
  })

  it('renderiza CardForm, RailSelector y TransactionViewer', () => {
    renderUi(<CheckoutPage />)

    expect(screen.getByLabelText(/datos de tarjeta/i)).toBeInTheDocument()
    expect(screen.getByRole('radiogroup', { name: /opciones de riel/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/log de transacción/i)).toBeInTheDocument()
  })
})
