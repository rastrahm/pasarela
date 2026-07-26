import { API_BASE_URL, GATEWAY_API_KEY } from '../config/env'
import { type CheckoutRequest } from '../schemas/checkout'
import { checkoutResultSchema, type CheckoutResult } from '../schemas/transaction'
import {
  GatewayApiError,
  parseGatewayErrorBody,
  resolveGatewayErrorMessage,
} from './errors'

export interface SubmitCheckoutOptions {
  baseUrl?: string
  apiKey?: string
  idempotencyKey?: string
}

/** Genera una clave de idempotencia única por intento de checkout (D9). */
export function createIdempotencyKey(): string {
  return crypto.randomUUID()
}

/**
 * Envía checkout al API Gateway (`POST /api/v1/checkout`).
 * El frontend nunca llama al Oracle directamente.
 */
export async function submitCheckout(
  request: CheckoutRequest,
  options: SubmitCheckoutOptions = {},
): Promise<CheckoutResult> {
  const baseUrl = options.baseUrl ?? API_BASE_URL
  const apiKey = options.apiKey ?? GATEWAY_API_KEY
  const idempotencyKey = options.idempotencyKey ?? createIdempotencyKey()

  const response = await fetch(`${baseUrl}/api/v1/checkout`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
      'Idempotency-Key': idempotencyKey,
    },
    body: JSON.stringify({
      amount: request.amount,
      currency: request.currency,
      funding_type: request.funding_type,
      card: request.card,
    }),
  })

  if (!response.ok) {
    const body = await parseGatewayErrorBody(response)
    throw new GatewayApiError(
      resolveGatewayErrorMessage(response.status, body),
      response.status,
      body.error_code,
    )
  }

  const json: unknown = await response.json()
  return checkoutResultSchema.parse(json)
}
