import { describe, expect, it } from 'vitest'

import { resolveGatewayErrorMessage } from './errors'

describe('resolveGatewayErrorMessage', () => {
  it('traduce 402 a fondos insuficientes', () => {
    expect(
      resolveGatewayErrorMessage(402, {
        error_code: 'INSUFFICIENT_FUNDS',
        message: 'fondos insuficientes',
      }),
    ).toBe('fondos insuficientes')
  })

  it('traduce 422 con tarjeta inválida', () => {
    expect(
      resolveGatewayErrorMessage(422, {
        error_code: 'INVALID_CARD',
        message: 'tarjeta inválida',
      }),
    ).toMatch(/tarjeta inválida/i)
  })

  it('traduce 503 a riel no disponible', () => {
    expect(resolveGatewayErrorMessage(503)).toMatch(/riel no disponible/i)
  })
})
