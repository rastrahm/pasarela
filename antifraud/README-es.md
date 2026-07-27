# Servicio Antifraude Simulado

Microservicio independiente que evalúa scoring de riesgo antes del hold del Oracle (UC-12, D11).

> Solo el **Oracle** invoca este servicio vía `POST /internal/v1/score`.

## Desarrollo

```bash
cd antifraud
cp .env.example .env
cargo run
```

Healthcheck: `GET http://localhost:8082/health`

## Endpoint interno

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `POST` | `/internal/v1/score` | `X-API-KEY` | Evalúa monto, token hash y reglas |

## Reglas simuladas

- Monto > `ANTIFRAUD_MAX_AMOUNT` → decline
- Token hash en `ANTIFRAUD_BLOCKED_TOKEN_HASHES` → decline
- Score >= `ANTIFRAUD_DECLINE_SCORE_THRESHOLD` → decline

## Tests

```bash
cargo test
```
