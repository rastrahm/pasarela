import { describe, expect, it } from 'vitest'

import {
  createGatewayApiError,
  mapGatewayErrorToUx,
  mapLocalCheckoutError,
  mapNetworkErrorToUx,
  resolveGatewayErrorMessage,
} from './errors'

describe('mapGatewayErrorToUx', () => {
  it('mapea 402 INSUFFICIENT_FUNDS con hint de riel', () => {
    const ux = mapGatewayErrorToUx(402, {
      error_code: 'INSUFFICIENT_FUNDS',
      message: 'fondos insuficientes',
    })

    expect(ux.kind).toBe('insufficient_funds')
    expect(ux.title).toMatch(/fondos/i)
    expect(ux.message).toBe('fondos insuficientes')
    expect(ux.hint).toMatch(/riel/i)
  })

  it('mapea 422 INVALID_CARD con hint de tarjeta demo', () => {
    const ux = mapGatewayErrorToUx(422, {
      error_code: 'INVALID_CARD',
      message: 'tarjeta inválida',
    })

    expect(ux.kind).toBe('invalid_card')
    expect(ux.message).toMatch(/tarjeta inválida/i)
    expect(ux.hint).toMatch(/datos de prueba/i)
  })

  it('mapea 422 INVALID_REQUEST genérico', () => {
    const ux = mapGatewayErrorToUx(422, {
      error_code: 'INVALID_REQUEST',
      message: 'monto inválido',
    })

    expect(ux.kind).toBe('invalid_request')
    expect(ux.message).toBe('monto inválido')
  })

  it('mapea 503 RAIL_UNAVAILABLE con hint de reintento', () => {
    const ux = mapGatewayErrorToUx(503, {
      error_code: 'RAIL_UNAVAILABLE',
      message: 'riel no disponible',
    })

    expect(ux.kind).toBe('rail_unavailable')
    expect(ux.message).toBe('riel no disponible')
    expect(ux.hint).toMatch(/otro riel/i)
  })

  it('mapea error de red', () => {
    const ux = mapNetworkErrorToUx(new TypeError('Failed to fetch'))

    expect(ux.kind).toBe('network')
    expect(ux.hint).toMatch(/VITE_API_BASE_URL/i)
  })

  it('compatibilidad resolveGatewayErrorMessage', () => {
    expect(
      resolveGatewayErrorMessage(503, {
        error_code: 'RAIL_UNAVAILABLE',
        message: 'riel no disponible',
      }),
    ).toBe('riel no disponible')
  })

  it('createGatewayApiError expone ux en la excepción', () => {
    const error = createGatewayApiError(402, {
      error_code: 'INSUFFICIENT_FUNDS',
      message: 'fondos insuficientes',
    })

    expect(error).toBeInstanceOf(Error)
    expect(error.ux.kind).toBe('insufficient_funds')
    expect(error.status).toBe(402)
  })

  it('mapLocalCheckoutError para validación previa', () => {
    const ux = mapLocalCheckoutError('Validá la tarjeta', 'Completá el formulario')

    expect(ux.kind).toBe('invalid_request')
    expect(ux.message).toBe('Validá la tarjeta')
    expect(ux.hint).toBe('Completá el formulario')
  })
})
