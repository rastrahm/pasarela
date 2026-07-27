> **Documentation / Documentación:** [Español (es)](README-es.md) · [English (en)](README-en.md)
>
# frontend — Pasarela Multi-Rail Checkout

Interfaz React del checkout multi-rail — **Fase 5 ✅** (acta: [Doc/Acta-Cierre-Fase-5.md](../Doc/Acta-Cierre-Fase-5.md)).

> El frontend **solo** comunica con el **API Gateway**. Nunca invoca al Oracle directamente.

## Requisitos

- Node.js 18+ (20+ recomendado para tooling futuro)
- pnpm 9+
- API Gateway en marcha (`cargo run -p api-gateway`)
- Oracle en marcha para checkout real (`cargo run` en `oracle/`)

## Configuración

```bash
cd frontend
cp .env.example .env.local
```

| Variable | Descripción |
|----------|-------------|
| `VITE_API_BASE_URL` | URL del Gateway (default `http://127.0.0.1:8080`) |
| `VITE_GATEWAY_API_KEY` | Debe coincidir con `GATEWAY_TEST_API_KEY` en `crates/api-gateway/.env` |

> El checkout en navegador requiere **CORS** en el Gateway (orígenes Vite `:5173`). Ver [Doc/Pruebas.md](../Doc/Pruebas.md).

## Desarrollo

```bash
pnpm install
pnpm dev          # http://localhost:5173
```

## Checkout manual

1. Arrancar Oracle y Gateway (ver READMEs respectivos).
2. Abrir `http://localhost:5173`.
3. Clic en **Usar datos de prueba** → **Validar tarjeta**.
4. Elegir riel (Banco / Binance / Solana).
5. **Confirmar pago**.
6. Verificar log en **Transacción** y comprobante (`ACH-*`, `CEX-*`, `SOL-*`).

## Tests

```bash
pnpm test         # watch mode
pnpm test:run     # CI — 78 tests (incl. contrato fixture)
pnpm lint
pnpm build
```

E2E Playwright (Fase 6): [tests/e2e/README.md](../tests/e2e/README.md) · Guía general: [Doc/Pruebas.md](../Doc/Pruebas.md)

## Estructura

```
src/
├── api/              # Cliente Gateway (checkout, transacciones, health)
├── components/       # CardForm, RailSelector, TransactionViewer, CheckoutErrorAlert
├── config/           # Variables VITE_* (env.ts)
├── hooks/            # useTransactionLog
├── pages/            # CheckoutPage
├── schemas/          # Zod — contrato Gateway + validación tarjeta
└── test/             # Helpers RTL (checkout-flow.ts)
```

## Endpoints consumidos

| Método | Ruta | Uso |
|--------|------|-----|
| `POST` | `/api/v1/checkout` | Checkout (principal) |
| `GET` | `/api/v1/transactions/{id}` | Cliente listo; UI pendiente |
| `GET` | `/health` | Healthcheck Gateway |
