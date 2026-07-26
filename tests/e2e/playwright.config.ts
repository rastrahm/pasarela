/**
 * Configuración Playwright — Fase 6.1 (checkout multi-rail).
 *
 * Prerrequisitos para tests contra stack real:
 * - API Gateway en VITE_API_BASE_URL (default http://127.0.0.1:8080)
 * - Oracle y simuladores según riel (ver tests/e2e/README.md)
 */
import { defineConfig, devices } from '@playwright/test'
import dotenv from 'dotenv'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const frontendDir = path.resolve(__dirname, '../../frontend')

dotenv.config({ path: path.join(frontendDir, '.env.local') })
dotenv.config({ path: path.join(frontendDir, '.env.example') })

const frontendPort = Number(process.env.E2E_FRONTEND_PORT ?? 5173)
const frontendBaseUrl =
  process.env.E2E_BASE_URL ?? `http://127.0.0.1:${frontendPort}`

export default defineConfig({
  testDir: './specs',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: [['list'], ['html', { open: 'never' }]],
  timeout: 60_000,
  expect: { timeout: 15_000 },
  use: {
    baseURL: frontendBaseUrl,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'pnpm dev --host 127.0.0.1 --port ' + frontendPort,
    cwd: frontendDir,
    url: frontendBaseUrl,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    env: {
      ...process.env,
      VITE_API_BASE_URL:
        process.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080',
      VITE_GATEWAY_API_KEY:
        process.env.VITE_GATEWAY_API_KEY ?? 'sk_test_change_me_32chars_min',
    },
  },
})
