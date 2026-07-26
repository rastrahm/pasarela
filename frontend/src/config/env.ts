/** URL base del API Gateway (sin barra final). */
export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080'

/** API key del comercio (`sk_test_...` / `sk_live_...`). */
export const GATEWAY_API_KEY =
  import.meta.env.VITE_GATEWAY_API_KEY ?? 'sk_test_change_me_32chars_min'
