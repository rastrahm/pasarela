import { describe, expect, it } from 'vitest'

import { DEFAULT_FUNDING_TYPE, fundingTypeSchema, RAIL_OPTIONS } from './funding'

describe('funding schema', () => {
  it('acepta los tres rieles del Gateway', () => {
    for (const option of RAIL_OPTIONS) {
      expect(fundingTypeSchema.safeParse(option.value).success).toBe(true)
    }
  })

  it('rechaza valores desconocidos', () => {
    expect(fundingTypeSchema.safeParse('stripe').success).toBe(false)
  })

  it('define banco tradicional como default', () => {
    expect(DEFAULT_FUNDING_TYPE).toBe('traditional_bank')
  })
})
