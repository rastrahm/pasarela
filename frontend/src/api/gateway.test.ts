import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

import { DEMO_CARD } from '../schemas/card'
import { submitCheckout } from './gateway'

const SUCCESS_BODY = {
  transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  status: 'settled',
  rail_used: 'traditional_bank',
  settlement_proof: 'ACH-MEM-001',
}

describe('submitCheckout', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn())
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('envía POST al Gateway con auth e idempotency', async () => {
    const fetchMock = vi.mocked(fetch).mockResolvedValue(
      new Response(JSON.stringify(SUCCESS_BODY), { status: 200 }),
    )

    await submitCheckout(
      {
        amount: 100,
        currency: 'USD',
        funding_type: 'traditional_bank',
        card: DEMO_CARD,
      },
      {
        baseUrl: 'http://gateway.test',
        apiKey: 'sk_test_demo',
        idempotencyKey: 'idem-123',
      },
    )

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0] ?? []
    expect(url).toBe('http://gateway.test/api/v1/checkout')
    expect(init?.method).toBe('POST')
    expect(init?.headers).toMatchObject({
      Authorization: 'Bearer sk_test_demo',
      'Idempotency-Key': 'idem-123',
    })

    const body = JSON.parse(String(init?.body))
    expect(body.amount).toBe(100)
    expect(body.funding_type).toBe('traditional_bank')
    expect(body.card.pan).toBe(DEMO_CARD.pan)
  })

  it('propaga errores HTTP del Gateway', async () => {
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({
          error_code: 'INSUFFICIENT_FUNDS',
          message: 'fondos insuficientes',
        }),
        { status: 402 },
      ),
    )

    await expect(
      submitCheckout(
        {
          amount: 100,
          currency: 'USD',
          funding_type: 'traditional_bank',
          card: DEMO_CARD,
        },
        { baseUrl: 'http://gateway.test', apiKey: 'sk_test_demo' },
      ),
    ).rejects.toMatchObject({ status: 402 })
  })
})
