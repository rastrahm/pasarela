# Simulated Antifraud Service

Standalone microservice that evaluates risk scoring before the Oracle hold (UC-12, D11).

> Only the **Oracle** invokes this service via `POST /internal/v1/score`.

## Development

```bash
cd antifraud
cp .env.example .env
cargo run
```

Healthcheck: `GET http://localhost:8082/health`

## Internal endpoint

| Method | Route | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/internal/v1/score` | `X-API-KEY` | Evaluates amount, token hash, and rules |

## Simulated rules

- Amount > `ANTIFRAUD_MAX_AMOUNT` → decline
- Token hash in `ANTIFRAUD_BLOCKED_TOKEN_HASHES` → decline
- Score >= `ANTIFRAUD_DECLINE_SCORE_THRESHOLD` → decline

## Tests

```bash
cargo test
```
