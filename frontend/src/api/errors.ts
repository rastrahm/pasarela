import type { GatewayErrorResponse } from '../schemas/gateway'

/** Error de red o HTTP al invocar el Gateway. */
export class GatewayApiError extends Error {
  readonly status: number
  readonly errorCode?: string

  constructor(message: string, status: number, errorCode?: string) {
    super(message)
    this.name = 'GatewayApiError'
    this.status = status
    this.errorCode = errorCode
  }
}

/**
 * Traduce códigos HTTP del Gateway a mensajes UX (402, 422, 503).
 */
export function resolveGatewayErrorMessage(
  status: number,
  body?: Partial<GatewayErrorResponse>,
): string {
  switch (status) {
    case 402:
      return body?.message ?? 'Fondos insuficientes en el riel seleccionado.'
    case 422:
      if (body?.error_code === 'INVALID_CARD') {
        return 'Tarjeta inválida. Verificá número, vencimiento y CVV.'
      }
      return body?.message ?? 'Solicitud inválida. Revisá los datos ingresados.'
    case 503:
      return body?.message ?? 'Riel no disponible. Probá otro método de liquidación.'
    case 401:
      return 'No autorizado. Verificá la API key del comercio.'
    case 409:
      return body?.message ?? 'Conflicto de idempotencia. Reintentá con otra clave.'
    default:
      return body?.message ?? 'Error al procesar el checkout.'
  }
}
