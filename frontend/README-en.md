# frontend — Pasarela Multi-Rail Checkout

Multi-rail checkout React interface — **Phase 5 ✅** (record: [Doc/Acta-Cierre-Fase-5-en.md](../Doc/Acta-Cierre-Fase-5-en.md)).

> The frontend **only** communicates with the **API Gateway**. It never invokes the Oracle directly.

## Requirements

- Node.js 18+ (20+ recommended for future tooling)
- pnpm 9+
- API Gateway running (`cargo run -p api-gateway`)
- Oracle running for real checkout (`cargo run` in `oracle/`)

## Configuration

```bash
cd frontend
cp .env.example .env.local
```

| Variable | Description |
|----------|-------------|
| `VITE_API_BASE_URL` | Gateway URL (default `http://127.0.0.1:8080`) |
| `VITE_GATEWAY_API_KEY` | Must match `GATEWAY_TEST_API_KEY` in `crates/api-gateway/.env` |

> Browser checkout requires **CORS** on the Gateway (Vite origins `:5173`). See [Doc/Pruebas-en.md](../Doc/Pruebas-en.md).

## Development

```bash
pnpm install
pnpm dev          # http://localhost:5173
```

## Manual checkout

1. Start Oracle and Gateway (see respective READMEs).
2. Open `http://localhost:5173`.
3. Click **Use test data** → **Validate card**.
4. Choose rail (Bank / Binance / Solana).
5. **Confirm payment**.
6. Verify log in **Transaction** and receipt (`ACH-*`, `CEX-*`, `SOL-*`).

## Tests

```bash
pnpm test         # watch mode
pnpm test:run     # CI — 78 tests (incl. fixture contract)
pnpm lint
pnpm build
```

E2E Playwright (Phase 6): [tests/e2e/README-en.md](../tests/e2e/README-en.md) · General guide: [Doc/Pruebas-en.md](../Doc/Pruebas-en.md)

## Structure

```
src/
├── api/              # Gateway client (checkout, transactions, health)
├── components/       # CardForm, RailSelector, TransactionViewer, CheckoutErrorAlert
├── config/           # VITE_* variables (env.ts)
├── hooks/            # useTransactionLog
├── pages/            # CheckoutPage
├── schemas/          # Zod — Gateway contract + card validation
└── test/             # RTL helpers (checkout-flow.ts)
```

## Consumed endpoints

| Method | Route | Purpose |
|--------|------|-----|
| `POST` | `/api/v1/checkout` | Checkout (primary) |
| `GET` | `/api/v1/transactions/{id}` | Client ready; UI pending |
| `GET` | `/health` | Gateway healthcheck |
