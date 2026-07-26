import type { GatewayErrorResponse } from '../schemas/gateway'

/** Categorías de error para estilos y acciones sugeridas en checkout. */
export type CheckoutErrorKind =
  | 'insufficient_funds'
  | 'invalid_card'
  | 'invalid_request'
  | 'rail_unavailable'
  | 'unauthorized'
  | 'conflict'
  | 'network'
  | 'unknown'

/** Presentación UX de un error de checkout. */
export interface CheckoutErrorUx {
  kind: CheckoutErrorKind
  status?: number
  errorCode?: string
  title: string
  message: string
  hint?: string
}

/** Error de red o HTTP al invocar el Gateway. */
export class GatewayApiError extends Error {
  readonly status: number
  readonly errorCode?: string
  readonly ux: CheckoutErrorUx

  constructor(ux: CheckoutErrorUx) {
    super(ux.message)
    this.name = 'GatewayApiError'
    this.ux = ux
    this.status = ux.status ?? 0
    this.errorCode = ux.errorCode
  }
}

const INSUFFICIENT_FUNDS_UX: Omit<CheckoutErrorUx, 'status' | 'errorCode' | 'message'> = {
  kind: 'insufficient_funds',
  title: 'Fondos insuficientes',
  hint: 'Probá otro riel de liquidación o reducí el monto del pago.',
}

const INVALID_CARD_UX: Omit<CheckoutErrorUx, 'status' | 'errorCode' | 'message'> = {
  kind: 'invalid_card',
  title: 'Tarjeta rechazada',
  hint: 'Usá «Usar datos de prueba» para cargar una tarjeta demo válida (Visa 4111…).',
}

const INVALID_REQUEST_UX: Omit<CheckoutErrorUx, 'status' | 'errorCode' | 'message'> = {
  kind: 'invalid_request',
  title: 'Datos incorrectos',
  hint: 'Revisá monto, moneda y que todos los campos estén completos.',
}

const RAIL_UNAVAILABLE_UX: Omit<CheckoutErrorUx, 'status' | 'errorCode' | 'message'> = {
  kind: 'rail_unavailable',
  title: 'Riel no disponible',
  hint: 'Elegí otro riel de liquidación o reintentá en unos segundos.',
}

/**
 * Mapea respuesta HTTP del Gateway a copy UX (402, 422, 503 prioritarios).
 */
export function mapGatewayErrorToUx(
  status: number,
  body?: Partial<GatewayErrorResponse>,
): CheckoutErrorUx {
  const errorCode = body?.error_code
  const serverMessage = body?.message

  switch (status) {
    case 402:
      return {
        ...INSUFFICIENT_FUNDS_UX,
        status,
        errorCode,
        message:
          serverMessage ??
          'No hay saldo suficiente en el riel seleccionado para completar el pago.',
      }

    case 422:
      if (errorCode === 'INVALID_CARD') {
        return {
          ...INVALID_CARD_UX,
          status,
          errorCode,
          message:
            serverMessage ??
            'Tarjeta inválida. Verificá número, vencimiento y CVV.',
        }
      }
      return {
        ...INVALID_REQUEST_UX,
        status,
        errorCode,
        message:
          serverMessage ?? 'La solicitud no pudo procesarse. Revisá los datos ingresados.',
      }

    case 503:
      return {
        ...RAIL_UNAVAILABLE_UX,
        status,
        errorCode: errorCode ?? 'RAIL_UNAVAILABLE',
        message:
          serverMessage ??
          'El riel de liquidación no está disponible en este momento.',
      }

    case 401:
      return {
        kind: 'unauthorized',
        status,
        errorCode,
        title: 'No autorizado',
        message: serverMessage ?? 'La API key del comercio fue rechazada.',
        hint: 'Verificá VITE_GATEWAY_API_KEY en frontend/.env.local.',
      }

    case 409:
      return {
        kind: 'conflict',
        status,
        errorCode,
        title: 'Conflicto de idempotencia',
        message: serverMessage ?? 'Esta clave de idempotencia ya se usó con otros datos.',
        hint: 'Reintentá el pago; se generará una clave nueva automáticamente.',
      }

    default:
      return {
        kind: 'unknown',
        status,
        errorCode,
        title: 'Error en el checkout',
        message: serverMessage ?? 'Ocurrió un error al procesar el pago.',
        hint: 'Si persiste, revisá los logs del Gateway y del Oracle.',
      }
  }
}

/** Mensaje plano — compatibilidad con código previo. */
export function resolveGatewayErrorMessage(
  status: number,
  body?: Partial<GatewayErrorResponse>,
): string {
  return mapGatewayErrorToUx(status, body).message
}

/** Errores de validación local antes de llamar al Gateway. */
export function mapLocalCheckoutError(message: string, hint?: string): CheckoutErrorUx {
  return {
    kind: 'invalid_request',
    title: 'Revisá el formulario',
    message,
    hint,
  }
}

/** Errores de red o fallos inesperados del fetch. */
export function mapNetworkErrorToUx(error: unknown): CheckoutErrorUx {
  if (error instanceof TypeError) {
    return {
      kind: 'network',
      title: 'Sin conexión al Gateway',
      message: 'No se pudo contactar al API Gateway.',
      hint: 'Verificá que el Gateway esté en marcha y que VITE_API_BASE_URL apunte a http://127.0.0.1:8080.',
    }
  }

  if (error instanceof Error) {
    return {
      kind: 'unknown',
      title: 'Error inesperado',
      message: error.message,
    }
  }

  return {
    kind: 'unknown',
    title: 'Error inesperado',
    message: 'Ocurrió un error desconocido durante el checkout.',
  }
}

/** Construye GatewayApiError a partir de la respuesta HTTP. */
export function createGatewayApiError(
  status: number,
  body?: Partial<GatewayErrorResponse>,
): GatewayApiError {
  return new GatewayApiError(mapGatewayErrorToUx(status, body))
}
