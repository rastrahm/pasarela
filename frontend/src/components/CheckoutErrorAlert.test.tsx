import { describe, expect, it } from 'vitest'

import { mapGatewayErrorToUx } from '../api/errors'
import { CheckoutErrorAlert } from './CheckoutErrorAlert'
import { renderUi, screen } from '../test/test-utils'

describe('CheckoutErrorAlert', () => {
  it('no renderiza sin error', () => {
    const { container } = renderUi(<CheckoutErrorAlert error={null} />)

    expect(container).toBeEmptyDOMElement()
  })

  it('muestra título, mensaje e hint para 402', () => {
    renderUi(
      <CheckoutErrorAlert
        error={mapGatewayErrorToUx(402, {
          error_code: 'INSUFFICIENT_FUNDS',
          message: 'fondos insuficientes',
        })}
      />,
    )

    expect(screen.getByRole('alert')).toHaveTextContent(/fondos insuficientes/i)
    expect(screen.getByRole('alert')).toHaveTextContent(/HTTP 402/)
    expect(screen.getByRole('alert')).toHaveTextContent(/INSUFFICIENT_FUNDS/)
  })

  it('muestra hint de tarjeta para 422 INVALID_CARD', () => {
    renderUi(
      <CheckoutErrorAlert
        error={mapGatewayErrorToUx(422, {
          error_code: 'INVALID_CARD',
          message: 'tarjeta inválida',
        })}
      />,
    )

    expect(screen.getByRole('alert')).toHaveTextContent(/datos de prueba/i)
  })

  it('muestra hint de otro riel para 503', () => {
    renderUi(
      <CheckoutErrorAlert
        error={mapGatewayErrorToUx(503, {
          error_code: 'RAIL_UNAVAILABLE',
          message: 'riel no disponible',
        })}
      />,
    )

    expect(screen.getByRole('alert')).toHaveTextContent(/otro riel/i)
    expect(screen.getByRole('alert')).toHaveTextContent(/HTTP 503/)
  })
})
