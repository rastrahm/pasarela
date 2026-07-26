/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** URL base del API Gateway — ver `frontend/.env.example`. */
  readonly VITE_API_BASE_URL?: string
  /** API key comercio (`sk_test_...`) — debe coincidir con GATEWAY_TEST_API_KEY. */
  readonly VITE_GATEWAY_API_KEY?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
