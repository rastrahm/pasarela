import { describe, expect, it } from 'vitest'

import {
  DEMO_CARD,
  detectCardBrand,
  luhnValid,
  validateCardPayload,
} from './card'

describe('card schema', () => {
  it('acepta la tarjeta demo del Gateway', () => {
    const result = validateCardPayload(DEMO_CARD)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data).toEqual(DEMO_CARD)
    }
  })

  it('rechaza PAN que no pasa Luhn', () => {
    const result = validateCardPayload({
      ...DEMO_CARD,
      pan: '4111111111111112',
    })

    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.errors.pan).toMatch(/luhn/i)
    }
  })

  it('rechaza marcas no soportadas', () => {
    const result = validateCardPayload({
      ...DEMO_CARD,
      pan: '6011111111111117',
    })

    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.errors.pan).toMatch(/marca/i)
    }
  })

  it('normaliza mes a dos dígitos', () => {
    const result = validateCardPayload({
      ...DEMO_CARD,
      expiry_month: '3',
    })

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.expiry_month).toBe('03')
    }
  })
})

describe('luhnValid', () => {
  it('valida Visa de prueba', () => {
    expect(luhnValid('4111111111111111')).toBe(true)
  })

  it('detecta Visa', () => {
    expect(detectCardBrand('4111111111111111')).toBe('visa')
  })
})
