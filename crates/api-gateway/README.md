# api-gateway

API Gateway y orquestador de checkout — **Fase 4, paso 4.5**.

> Expone la API pública del comercio. Solo el Gateway invoca al Oracle; el frontend nunca accede al Oracle directamente.

## Endpoints

| Método | Ruta | Auth | Estado |
|--------|------|------|--------|
| `GET` | `/health` | — | ✅ |
| `POST` | `/api/v1/checkout` | API key (4.11) + Idempotency-Key (4.10) | ✅ 4.6–4.7 |
| `GET` | `/api/v1/transactions/{id}` | API key comercio | Stub → 4.8 |

## Arranque

```bash
cp .env.example .env
# Editar ORACLE_API_KEY (debe coincidir con oracle/.env)

cargo run -p api-gateway
curl http://127.0.0.1:8080/health
```

## Tests

```bash
cargo test -p api-gateway
```

## Dependencias internas

- `domain` — tipos compartidos
- `oracle-client` — cliente HTTP hacia Oracle
- `rail-switcher` — selección de riel + fallback D3 (paso 4.7)
- `settlement-adapters` — liquidación por riel
