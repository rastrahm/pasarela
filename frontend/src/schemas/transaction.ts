import { z } from 'zod'

import { fundingTypeSchema, RAIL_OPTIONS, type FundingType } from './funding'

export const transactionStatusSchema = z.enum([
  'pending',
  'authorized',
  'held',
  'settled',
  'failed',
  'reversed',
])

export type TransactionStatus = z.infer<typeof transactionStatusSchema>

export type LogLevel = 'info' | 'success' | 'warning' | 'error'

export interface TransactionLogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  message: string
}

/** Respuesta de checkout alineada con `CheckoutResponse` del Gateway. */
export interface CheckoutResult {
  transaction_id: string
  status: TransactionStatus
  rail_used: FundingType
  settlement_proof: string | null
}

export const checkoutResultSchema = z.object({
  transaction_id: z.string().uuid(),
  status: transactionStatusSchema,
  rail_used: fundingTypeSchema,
  settlement_proof: z.string().nullable(),
})

/** Mensajes estándar del flujo UC-10. */
export const TRANSACTION_LOG_MESSAGES = {
  validatingCard: 'Validando tarjeta…',
  authorizing: 'Autorizando con Oracle…',
  holdConfirmed: 'Hold confirmado',
  holdRejected: 'Hold rechazado',
  settling: (railLabel: string) => `Liquidando en ${railLabel}…`,
  settled: 'Liquidación completada',
  failed: 'Checkout fallido',
} as const

let logSequence = 0

/** Crea una entrada de log con identificador único. */
export function createLogEntry(
  message: string,
  level: LogLevel = 'info',
  timestamp: Date = new Date(),
): TransactionLogEntry {
  logSequence += 1
  return {
    id: `log-${logSequence}-${timestamp.getTime()}`,
    timestamp,
    level,
    message,
  }
}

/** Devuelve la etiqueta legible de un riel. */
export function getRailLabel(rail: FundingType): string {
  return RAIL_OPTIONS.find((option) => option.value === rail)?.label ?? rail
}

/** Etiqueta del comprobante según el riel usado. */
export function getSettlementProofLabel(rail: FundingType): string {
  switch (rail) {
    case 'traditional_bank':
      return 'Referencia bancaria'
    case 'binance_cex':
      return 'Order ID'
    case 'solana_wallet':
      return 'Tx Signature'
    default:
      return 'Comprobante'
  }
}

const STATUS_LABELS: Record<TransactionStatus, string> = {
  pending: 'Pendiente',
  authorized: 'Autorizada',
  held: 'Hold activo',
  settled: 'Liquidada',
  failed: 'Fallida',
  reversed: 'Revertida',
}

/** Traduce el estado de transacción para la UI. */
export function getStatusLabel(status: TransactionStatus): string {
  return STATUS_LABELS[status]
}

/** Genera entradas de log a partir del resultado final de checkout. */
export function buildCheckoutLogEntries(result: CheckoutResult): TransactionLogEntry[] {
  const railLabel = getRailLabel(result.rail_used)
  const entries: TransactionLogEntry[] = [
    createLogEntry(TRANSACTION_LOG_MESSAGES.validatingCard),
    createLogEntry(TRANSACTION_LOG_MESSAGES.authorizing),
  ]

  if (result.status === 'failed') {
    entries.push(
      createLogEntry(TRANSACTION_LOG_MESSAGES.holdRejected, 'error'),
      createLogEntry(TRANSACTION_LOG_MESSAGES.failed, 'error'),
    )
    return entries
  }

  entries.push(createLogEntry(TRANSACTION_LOG_MESSAGES.holdConfirmed, 'success'))
  entries.push(createLogEntry(TRANSACTION_LOG_MESSAGES.settling(railLabel)))

  if (result.status === 'settled') {
    entries.push(createLogEntry(TRANSACTION_LOG_MESSAGES.settled, 'success'))
  }

  return entries
}

/** Reinicia el contador interno de IDs (solo tests). */
export function resetLogSequenceForTests(): void {
  logSequence = 0
}
