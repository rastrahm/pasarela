import { describe, expect, it } from 'vitest'

import {
  DEFAULT_API_BASE_URL,
  DEFAULT_GATEWAY_API_KEY,
  normalizeBaseUrl,
  resolveAppEnv,
} from './env'

describe('app env', () => {
  it('usa defaults cuando no hay variables Vite', () => {
    const env = resolveAppEnv({})

    expect(env.apiBaseUrl).toBe(DEFAULT_API_BASE_URL)
    expect(env.gatewayApiKey).toBe(DEFAULT_GATEWAY_API_KEY)
    expect(env.usingDefaults.apiBaseUrl).toBe(true)
    expect(env.usingDefaults.gatewayApiKey).toBe(true)
  })

  it('lee VITE_API_BASE_URL y normaliza barra final', () => {
    const env = resolveAppEnv({
      VITE_API_BASE_URL: 'http://127.0.0.1:8080/',
      VITE_GATEWAY_API_KEY: 'sk_test_custom_key_123456',
    })

    expect(env.apiBaseUrl).toBe('http://127.0.0.1:8080')
    expect(env.gatewayApiKey).toBe('sk_test_custom_key_123456')
    expect(env.usingDefaults.apiBaseUrl).toBe(false)
  })

  it('normalizeBaseUrl elimina slashes trailing', () => {
    expect(normalizeBaseUrl('http://gateway.test///')).toBe('http://gateway.test')
  })

  it('ignora URL inválida y cae al default', () => {
    const env = resolveAppEnv({
      VITE_API_BASE_URL: 'not-a-url',
    })

    expect(env.apiBaseUrl).toBe(DEFAULT_API_BASE_URL)
    expect(env.usingDefaults.apiBaseUrl).toBe(true)
  })
})
