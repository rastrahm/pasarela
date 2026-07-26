import { describe, expect, it } from 'vitest'

import {
  checkoutRequestSchema,
  checkoutResponseSchema,
  GATEWAY_CHECKOUT_FIXTURE,
  GATEWAY_CHECKOUT_RESPONSE_FIXTURE,
  gatewayErrorResponseSchema,
  healthResponseSchema,
  parseCheckoutRequest,
  parseCheckoutResponse,
  parseGatewayErrorResponse,
  parseHealthResponse,
  parseTransactionResponse,
  toCheckoutRequestBody,
  transactionResponseSchema,
} from './gateway'

describe('gateway schemas', () => {
  it('valida CheckoutRequest con fixture E2E del Gateway', () => {
    const request = parseCheckoutRequest(GATEWAY_CHECKOUT_FIXTURE)

    expect(request.currency).toBe('USD')
    expect(checkoutRequestSchema.safeParse(request).success).toBe(true)
    expect(toCheckoutRequestBody(request).card.pan).toBe('4111111111111111')
  })

  it('rechaza montos no positivos en checkout', () => {
    expect(() =>
      parseCheckoutRequest({
        ...GATEWAY_CHECKOUT_FIXTURE,
        amount: 0,
      }),
    ).toThrow()
  })

  it('valida CheckoutResponse settled', () => {
    expect(
      checkoutResponseSchema.safeParse(GATEWAY_CHECKOUT_RESPONSE_FIXTURE).success,
    ).toBe(true)
    expect(parseCheckoutResponse(GATEWAY_CHECKOUT_RESPONSE_FIXTURE).status).toBe(
      'settled',
    )
  })

  it('valida TransactionResponse con riel opcional', () => {
    const payload = {
      transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
      status: 'pending',
      rail_used: null,
      settlement_proof: null,
    }

    expect(transactionResponseSchema.safeParse(payload).success).toBe(true)
    expect(parseTransactionResponse(payload).rail_used).toBeNull()
  })

  it('valida HealthResponse', () => {
    const payload = { status: 'ok', service: 'api-gateway' }

    expect(healthResponseSchema.safeParse(payload).success).toBe(true)
    expect(parseHealthResponse(payload).service).toBe('api-gateway')
  })

  it('valida ErrorResponse del Gateway', () => {
    const payload = {
      error_code: 'INSUFFICIENT_FUNDS',
      message: 'fondos insuficientes',
    }

    expect(gatewayErrorResponseSchema.safeParse(payload).success).toBe(true)
    expect(parseGatewayErrorResponse(payload).error_code).toBe('INSUFFICIENT_FUNDS')
  })

  it('acepta códigos de error desconocidos como string', () => {
    const payload = {
      error_code: 'CUSTOM_ERROR',
      message: 'detalle',
    }

    expect(parseGatewayErrorResponse(payload).error_code).toBe('CUSTOM_ERROR')
  })
})
