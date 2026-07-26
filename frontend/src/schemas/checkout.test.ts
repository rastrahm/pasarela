import { describe, expect, it } from 'vitest'

import { DEMO_CARD } from './card'
import { checkoutRequestSchema, parseCheckoutRequest } from './checkout'

describe('checkout schema', () => {
  it('valida solicitud alineada con el Gateway', () => {
    const request = parseCheckoutRequest({
      amount: 100,
      currency: 'usd',
      funding_type: 'traditional_bank',
      card: DEMO_CARD,
    })

    expect(request.currency).toBe('USD')
    expect(checkoutRequestSchema.safeParse(request).success).toBe(true)
  })

  it('rechaza montos no positivos', () => {
    expect(() =>
      parseCheckoutRequest({
        amount: 0,
        currency: 'USD',
        card: DEMO_CARD,
      }),
    ).toThrow()
  })
})
