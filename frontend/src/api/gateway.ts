import { API_BASE_URL, GATEWAY_API_KEY } from '../config/env'
import {
  parseCheckoutResponse,
  parseGatewayErrorResponse,
  parseHealthResponse,
  parseTransactionResponse,
  toCheckoutRequestBody,
  type CheckoutRequest,
  type CheckoutResponse,
  type GatewayErrorResponse,
  type TransactionResponse,
} from '../schemas/gateway'
import { createGatewayApiError } from './errors'

export interface GatewayClientOptions {
  baseUrl?: string
  apiKey?: string
  idempotencyKey?: string
}

/** Genera una clave de idempotencia única por intento de checkout (D9). */
export function createIdempotencyKey(): string {
  return crypto.randomUUID()
}

function resolveClientOptions(options: GatewayClientOptions = {}) {
  return {
    baseUrl: options.baseUrl ?? API_BASE_URL,
    apiKey: options.apiKey ?? GATEWAY_API_KEY,
    idempotencyKey: options.idempotencyKey ?? createIdempotencyKey(),
  }
}

async function parseErrorResponse(response: Response): Promise<never> {
  let body: Partial<GatewayErrorResponse> | undefined
  try {
    const json: unknown = await response.json()
    body = parseGatewayErrorResponse(json)
  } catch {
    body = undefined
  }

  throw createGatewayApiError(response.status, body)
}

/**
 * Envía checkout al API Gateway (`POST /api/v1/checkout`).
 * El frontend nunca llama al Oracle directamente.
 */
export async function submitCheckout(
  request: CheckoutRequest,
  options: GatewayClientOptions = {},
): Promise<CheckoutResponse> {
  const { baseUrl, apiKey, idempotencyKey } = resolveClientOptions(options)

  const response = await fetch(`${baseUrl}/api/v1/checkout`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
      'Idempotency-Key': idempotencyKey,
    },
    body: JSON.stringify(toCheckoutRequestBody(request)),
  })

  if (!response.ok) {
    throw await parseErrorResponse(response)
  }

  const json: unknown = await response.json()
  return parseCheckoutResponse(json)
}

/** Consulta una transacción (`GET /api/v1/transactions/{id}`). */
export async function fetchTransaction(
  transactionId: string,
  options: Omit<GatewayClientOptions, 'idempotencyKey'> = {},
): Promise<TransactionResponse> {
  const { baseUrl, apiKey } = resolveClientOptions(options)

  const response = await fetch(`${baseUrl}/api/v1/transactions/${transactionId}`, {
    headers: {
      Authorization: `Bearer ${apiKey}`,
    },
  })

  if (!response.ok) {
    throw await parseErrorResponse(response)
  }

  const json: unknown = await response.json()
  return parseTransactionResponse(json)
}

/** Healthcheck del Gateway (`GET /health`). */
export async function fetchGatewayHealth(
  options: Pick<GatewayClientOptions, 'baseUrl'> = {},
): Promise<{ status: string; service: string }> {
  const baseUrl = options.baseUrl ?? API_BASE_URL

  const response = await fetch(`${baseUrl}/health`)
  if (!response.ok) {
    throw await parseErrorResponse(response)
  }

  const json: unknown = await response.json()
  return parseHealthResponse(json)
}
