# Oracle de Autorización

Microservicio **independiente** de la pasarela. Simula la red procesadora (Visa/Mastercard), valida tarjetas y gestiona holds de fondos por riel.

> Solo el **API Gateway** puede invocar este servicio, vía red interna y `X-API-KEY`.

## Requisitos

- Rust 1.75+
- Variables de entorno (ver `.env.example`)

## Configuración

```bash
cp .env.example .env
# Editar ORACLE_API_KEY y ORACLE_ALLOWED_CALLERS
```

## Desarrollo

```bash
cd oracle
cargo run
```

Healthcheck: `GET http://localhost:8081/health`

## Endpoints internos (requieren autenticación)

| Método | Ruta | Descripción |
|--------|------|-------------|
| `POST` | `/internal/v1/authorize` | Validar tarjeta + evaluar fondos + crear hold |
| `POST` | `/internal/v1/hold/release` | Liberar hold si falla el settlement |
| `GET` | `/health` | Healthcheck (sin auth) |

## Seguridad

- Red privada: no exponer a Internet
- Header `X-API-KEY` obligatorio en `/internal/v1/*`
- Allowlist de IP en `ORACLE_ALLOWED_CALLERS`
- Rate limiting configurable
- PAN tokenizado en memoria; nunca persistido

## Tests

```bash
cargo test
```

- `tests/health_integration.rs` — healthcheck público
- `tests/auth_security.rs` — rechazo sin API key / key inválida

## Despliegue

```bash
docker build -t oracle-authorization .
docker run --env-file .env -p 8081:8081 oracle-authorization
```

En producción, conectar el contenedor solo a la red Docker/VPC del Gateway.

## Relación con la pasarela

El Gateway consume este servicio mediante el crate `pasarela/crates/oracle-client/` (pendiente de Fase 4). No hay dependencia Cargo directa entre proyectos.
