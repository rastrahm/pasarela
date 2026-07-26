import { z } from 'zod'

import { cardPayloadSchema } from './card'
import { fundingTypeSchema } from './funding'

/**
 * Contrato HTTP del API Gateway — espejo de `crates/api-gateway/src/routes/dto.rs`
 * y `crates/api-gateway/src/error.rs`.
 */

/** Estados del ciclo de vida (`domain::TransactionStatus`, snake_case). */
export const transactionStatusSchema = z.enum([
  'pending',
  'authorized',
  'held',
  'settled',
  'failed',
  'reversed',
])

export type TransactionStatus = z.infer<typeof transactionStatusSchema>

/** Códigos de error expuestos al comercio (`GatewayError::error_code`). */
export const gatewayErrorCodeSchema = z.enum([
  'UNAUTHORIZED',
  'INSUFFICIENT_FUNDS',
  'INVALID_CARD',
  'INVALID_REQUEST',
  'CONFLICT',
  'NOT_FOUND',
  'RAIL_UNAVAILABLE',
  'INTERNAL_ERROR',
  'NOT_IMPLEMENTED',
])

export type GatewayErrorCode = z.infer<typeof gatewayErrorCodeSchema>

/** Datos de tarjeta en checkout (`CheckoutCardPayload`). */
export const checkoutCardPayloadSchema = cardPayloadSchema

export type CheckoutCardPayload = z.output<typeof checkoutCardPayloadSchema>
export type CheckoutCardPayloadInput = z.input<typeof checkoutCardPayloadSchema>

/** Solicitud `POST /api/v1/checkout` (`CheckoutRequest`). */
export const checkoutRequestSchema = z.object({
  amount: z.number().positive('El monto debe ser mayor a 0'),
  currency: z
    .string()
    .trim()
    .length(3, 'La moneda debe tener 3 letras (ISO 4217)')
    .transform((value) => value.toUpperCase()),
  funding_type: fundingTypeSchema.optional(),
  card: checkoutCardPayloadSchema,
})

export type CheckoutRequest = z.output<typeof checkoutRequestSchema>
export type CheckoutRequestInput = z.input<typeof checkoutRequestSchema>

/** Respuesta exitosa de checkout (`CheckoutResponse`). */
export const checkoutResponseSchema = z.object({
  transaction_id: z.string().uuid(),
  status: transactionStatusSchema,
  rail_used: fundingTypeSchema,
  settlement_proof: z.string().nullable(),
})

export type CheckoutResponse = z.infer<typeof checkoutResponseSchema>

/** Respuesta `GET /api/v1/transactions/{id}` (`TransactionResponse`). */
export const transactionResponseSchema = z.object({
  transaction_id: z.string().uuid(),
  status: transactionStatusSchema,
  rail_used: fundingTypeSchema.nullable(),
  settlement_proof: z.string().nullable(),
})

export type TransactionResponse = z.infer<typeof transactionResponseSchema>

/** Healthcheck `GET /health` (`HealthResponse`). */
export const healthResponseSchema = z.object({
  status: z.string(),
  service: z.string(),
})

export type HealthResponse = z.infer<typeof healthResponseSchema>

/** Cuerpo JSON de error (`ErrorResponse`). */
export const gatewayErrorResponseSchema = z.object({
  error_code: z.union([gatewayErrorCodeSchema, z.string()]),
  message: z.string(),
})

export type GatewayErrorResponse = z.infer<typeof gatewayErrorResponseSchema>

/** Payload de checkout usado en tests E2E del Gateway. */
export const GATEWAY_CHECKOUT_FIXTURE = {
  amount: 100,
  currency: 'USD',
  funding_type: 'traditional_bank',
  card: {
    pan: '4111111111111111',
    expiry_month: '12',
    expiry_year: '2030',
    cvv: '123',
    cardholder: 'Demo User',
  },
} as const satisfies CheckoutRequestInput

/** Respuesta de checkout settled típica en integración. */
export const GATEWAY_CHECKOUT_RESPONSE_FIXTURE = {
  transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  status: 'settled',
  rail_used: 'traditional_bank',
  settlement_proof: 'ACH-MEM-001',
} as const satisfies CheckoutResponse

export function parseCheckoutRequest(input: CheckoutRequestInput): CheckoutRequest {
  return checkoutRequestSchema.parse(input)
}

export function parseCheckoutResponse(input: unknown): CheckoutResponse {
  return checkoutResponseSchema.parse(input)
}

export function parseTransactionResponse(input: unknown): TransactionResponse {
  return transactionResponseSchema.parse(input)
}

export function parseHealthResponse(input: unknown): HealthResponse {
  return healthResponseSchema.parse(input)
}

export function parseGatewayErrorResponse(input: unknown): GatewayErrorResponse {
  return gatewayErrorResponseSchema.parse(input)
}

/** Serializa la solicitud validada al JSON enviado al Gateway. */
export function toCheckoutRequestBody(request: CheckoutRequest): {
  amount: number
  currency: string
  funding_type?: CheckoutRequest['funding_type']
  card: CheckoutCardPayload
} {
  return {
    amount: request.amount,
    currency: request.currency,
    funding_type: request.funding_type,
    card: request.card,
  }
}

/** @deprecated Usar `CheckoutResponse`. */
export type CheckoutResult = CheckoutResponse

/** @deprecated Usar `checkoutResponseSchema`. */
export const checkoutResultSchema = checkoutResponseSchema

/** @deprecated Usar `parseCheckoutResponse`. */
export const parseCheckoutResult = parseCheckoutResponse

/** @deprecated Usar `GatewayErrorResponse`. */
export type GatewayErrorBody = GatewayErrorResponse
