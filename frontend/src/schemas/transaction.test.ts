import { describe, expect, it, beforeEach } from 'vitest'

import type { FundingType } from './funding'
import {
  buildCheckoutLogEntries,
  createLogEntry,
  getSettlementProofLabel,
  getStatusLabel,
  resetLogSequenceForTests,
  TRANSACTION_LOG_MESSAGES,
} from './transaction'
import {
  GATEWAY_CHECKOUT_RESPONSE_FIXTURE,
  checkoutResponseSchema,
  parseCheckoutResponse,
} from './gateway'

describe('transaction schema', () => {
  beforeEach(() => {
    resetLogSequenceForTests()
  })

  it('valida una respuesta de checkout del Gateway', () => {
    expect(
      checkoutResponseSchema.safeParse(GATEWAY_CHECKOUT_RESPONSE_FIXTURE).success,
    ).toBe(true)
    expect(parseCheckoutResponse(GATEWAY_CHECKOUT_RESPONSE_FIXTURE).status).toBe(
      'settled',
    )
  })

  it('genera entradas de log para checkout exitoso', () => {
    const entries = buildCheckoutLogEntries({
      ...GATEWAY_CHECKOUT_RESPONSE_FIXTURE,
      rail_used: 'binance_cex',
      settlement_proof: 'CEX-MEM-001',
    })

    expect(entries[0]?.message).toBe(TRANSACTION_LOG_MESSAGES.validatingCard)
    expect(entries[entries.length - 1]?.message).toBe(TRANSACTION_LOG_MESSAGES.settled)
    expect(entries.some((entry) => entry.message.includes('Binance CEX'))).toBe(true)
  })

  it('genera entradas de error para checkout fallido', () => {
    const entries = buildCheckoutLogEntries({
      transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
      status: 'failed',
      rail_used: 'solana_wallet',
      settlement_proof: null,
    })

    const last = entries[entries.length - 1]
    expect(last?.level).toBe('error')
    expect(last?.message).toBe(TRANSACTION_LOG_MESSAGES.failed)
  })

  it('traduce estados de transacción', () => {
    expect(getStatusLabel('settled')).toBe('Liquidada')
  })

  it('asigna etiqueta de comprobante por riel', () => {
    const cases: Array<[FundingType, string]> = [
      ['traditional_bank', 'Referencia bancaria'],
      ['binance_cex', 'Order ID'],
      ['solana_wallet', 'Tx Signature'],
    ]

    for (const [rail, label] of cases) {
      expect(getSettlementProofLabel(rail)).toBe(label)
    }
  })

  it('crea entradas con id único', () => {
    const first = createLogEntry('Paso 1')
    const second = createLogEntry('Paso 2')

    expect(first.id).not.toBe(second.id)
  })
})
