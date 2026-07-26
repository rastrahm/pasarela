/**
 * Variables de entorno del frontend (Vite — prefijo `VITE_`).
 *
 * Copiar `frontend/.env.example` → `frontend/.env.local` y alinear valores
 * con `crates/api-gateway/.env` (`GATEWAY_PORT`, `GATEWAY_TEST_API_KEY`).
 */

import { z } from 'zod'

/** URL por defecto del API Gateway (`GATEWAY_PORT=8080`). */
export const DEFAULT_API_BASE_URL = 'http://127.0.0.1:8080'

/** API key demo — debe coincidir con `GATEWAY_TEST_API_KEY` del Gateway. */
export const DEFAULT_GATEWAY_API_KEY = 'sk_test_change_me_32chars_min'

const viteEnvSchema = z.object({
  VITE_API_BASE_URL: z.string().url().optional(),
  VITE_GATEWAY_API_KEY: z.string().min(16).optional(),
})

export interface AppEnv {
  apiBaseUrl: string
  gatewayApiKey: string
  usingDefaults: {
    apiBaseUrl: boolean
    gatewayApiKey: boolean
  }
}

/** Elimina barras finales de la URL base del Gateway. */
export function normalizeBaseUrl(url: string): string {
  return url.trim().replace(/\/+$/, '')
}

/**
 * Resuelve configuración desde variables Vite (testeable sin `import.meta`).
 */
export function resolveAppEnv(
  raw: Record<string, string | undefined> = import.meta.env,
): AppEnv {
  const parsed = viteEnvSchema.safeParse(raw)

  const rawBaseUrl = parsed.success ? parsed.data.VITE_API_BASE_URL : undefined
  const rawApiKey = parsed.success ? parsed.data.VITE_GATEWAY_API_KEY : undefined

  const apiBaseUrl = normalizeBaseUrl(rawBaseUrl ?? DEFAULT_API_BASE_URL)
  const gatewayApiKey = rawApiKey ?? DEFAULT_GATEWAY_API_KEY

  return {
    apiBaseUrl,
    gatewayApiKey,
    usingDefaults: {
      apiBaseUrl: !rawBaseUrl,
      gatewayApiKey: !rawApiKey,
    },
  }
}

/** Configuración activa de la app (único punto de lectura de env). */
export const appEnv = resolveAppEnv()

/** URL base del API Gateway (sin barra final). */
export const API_BASE_URL = appEnv.apiBaseUrl

/** API key del comercio (`Authorization: Bearer sk_*`). */
export const GATEWAY_API_KEY = appEnv.gatewayApiKey

/** Indica si se usa la API key placeholder del `.env.example`. */
export function isPlaceholderApiKey(apiKey: string = GATEWAY_API_KEY): boolean {
  return apiKey === DEFAULT_GATEWAY_API_KEY
}
