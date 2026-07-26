import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

import { DEMO_CARD } from '../schemas/card'
import { fetchGatewayHealth, fetchTransaction, submitCheckout } from './gateway'

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

  it('consulta transacción con auth', async () => {
    const fetchMock = vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({
          transaction_id: SUCCESS_BODY.transaction_id,
          status: 'settled',
          rail_used: 'traditional_bank',
          settlement_proof: 'ACH-MEM-001',
        }),
        { status: 200 },
      ),
    )

    const tx = await fetchTransaction(SUCCESS_BODY.transaction_id, {
      baseUrl: 'http://gateway.test',
      apiKey: 'sk_test_demo',
    })

    expect(tx.status).toBe('settled')
    expect(fetchMock).toHaveBeenCalledWith(
      `http://gateway.test/api/v1/transactions/${SUCCESS_BODY.transaction_id}`,
      expect.objectContaining({
        headers: { Authorization: 'Bearer sk_test_demo' },
      }),
    )
  })

  it('consulta health del Gateway', async () => {
    vi.mocked(fetch).mockResolvedValue(
      new Response(
        JSON.stringify({ status: 'ok', service: 'api-gateway' }),
        { status: 200 },
      ),
    )

    const health = await fetchGatewayHealth({ baseUrl: 'http://gateway.test' })
    expect(health.service).toBe('api-gateway')
  })
})
