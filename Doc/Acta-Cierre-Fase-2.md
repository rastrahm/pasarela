> **Documentation / Documentación:** [Español (es)](Acta-Cierre-Fase-2-es.md) · [English (en)](Acta-Cierre-Fase-2-en.md)
>
# Acta de Cierre — Fase 2 (Oracle de autorización)

> Gate **2.11** · Verificación técnica y confirmación stakeholder  
> Fecha: **2026-07-25**  
> Proyecto: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaración

Se declara **cerrada la Fase 2 — Oracle de autorización (servicio independiente)**, con el microservicio `oracle/` operativo, contrato API v1, cliente Gateway (`oracle-client`), integración antifraude y suite de tests de seguridad verificada.

Queda autorizado avanzar hacia **Fase 3 (Solana/Anchor)** y/o **Fase 4 (API Gateway)**, según prioridad del equipo (pueden paralelizarse parcialmente).

---

## 2. Entregables verificados

| Entregable | Ubicación | Estado |
|------------|-----------|--------|
| Contrato API v1 | `oracle/docs/API-v1.md`, `oracle/openapi/v1.yaml` | ✅ |
| Persistencia holds + audit log | `oracle/migrations/001_init.sql`, `oracle/src/persistence/` | ✅ |
| TTL de holds | `oracle/src/ttl/` | ✅ |
| Release de holds | `POST /internal/v1/hold/release` | ✅ |
| Rate limiting | `oracle/src/auth/rate_limit.rs` | ✅ |
| Adapters de rieles | `oracle/src/rail_adapters/`, `binance-sim/` | ✅ |
| Cliente antifraude | `oracle/src/antifraud_client/`, `antifraud/` | ✅ |
| Logging sin PII | `oracle/src/logging/` | ✅ |
| Crate `oracle-client` | `crates/oracle-client/` | ✅ |
| Tests Gateway ↔ Oracle | `oracle/tests/gateway_contract.rs` | ✅ |
| Variables de entorno | `oracle/.env.example` | ✅ |

---

## 3. Checklist de pasos (Plan §6)

| # | Paso | Resultado |
|---|------|-----------|
| 2.1 | Contrato API v1 | ✅ |
| 2.2 | Persistencia de holds | ✅ |
| 2.3 | Expiración TTL | ✅ |
| 2.4 | `POST /hold/release` | ✅ |
| 2.5 | Rate limiting | ✅ |
| 2.6 | Consultas por riel | ✅ |
| 2.7 | Antifraude simulado | ✅ |
| 2.8 | Logging estructurado sin PII | ✅ |
| 2.9 | Tests de seguridad ampliados | ✅ |
| 2.10 | Crate `oracle-client` | ✅ |
| 2.11 | Tests contrato Gateway ↔ Oracle | ✅ |

---

## 4. Criterios de aceptación (gate)

| Criterio | Verificación | Resultado |
|----------|--------------|-----------|
| `cargo test` verde en `oracle/` | 67 tests (`--test-threads=1`) | ✅ |
| `cargo test` verde en `oracle-client` | 13 tests | ✅ |
| PAN nunca persiste | `approved_authorization_persists_token_hash_not_pan`, migraciones | ✅ |
| Logs/audit sin PII | `logging_no_pii` (2 tests) | ✅ |
| Sin API key → 401 | `unauthorized_authorize_does_not_create_hold`, `gateway_contract` | ✅ |
| IP inválida → 403 | `forbidden_ip_does_not_create_hold`, `gateway_contract` | ✅ |
| Hold se crea y libera | `hold_persistence`, `gateway_client_authorize_and_release_full_flow` | ✅ |
| Fail closed (fondos, antifraude, riel) | `security_fail_closed` (10 tests) | ✅ |
| Confirmación stakeholder | Gate formal | ✅ 2026-07-25 |

### Nota sobre `hold/consume`

El endpoint `POST /internal/v1/hold/consume` queda planificado para **Fase 4** (Gateway, API v1.1). La persistencia ya soporta estado `consumed` vía `update_status_in_tx`; no bloquea el cierre de Fase 2.

---

## 5. Verificación técnica ejecutada

```bash
# Oracle (requiere PostgreSQL)
cd oracle
ORACLE_DATABASE_URL='postgres://postgres:postgre@localhost:5432/oracle' \
  cargo test -- --test-threads=1

# Cliente Gateway
cargo test -p oracle-client

# Antifraude (servicio auxiliar D11)
cd antifraud && cargo test
```

| Componente | Tests | Resultado |
|------------|-------|-----------|
| `oracle/` (unit) | 30 | ✅ |
| `oracle/` (integración) | 37 | ✅ |
| `oracle-client` | 13 | ✅ |
| `antifraud` | 7 | ✅ |
| **Total Fase 2** | **87** | ✅ |

### Suites de integración Oracle

| Archivo | Tests | Área |
|---------|-------|------|
| `security_fail_closed.rs` | 10 | Allowlist, rate limit, fail closed |
| `gateway_contract.rs` | 7 | Contrato Gateway ↔ Oracle |
| `hold_persistence.rs` | 3 | Creación, release, fondos |
| `logging_no_pii.rs` | 2 | Ausencia PAN/CVV en logs |
| `rail_adapters_integration.rs` | 2 | Rieles HTTP/RPC |
| `antifraud_integration.rs` | 3 | Fail closed antifraude |
| `rate_limit_integration.rs` | 2 | 429 por API key + IP |
| `auth_security.rs` | 4 | UC-11 básico |
| `health_integration.rs` | 1 | Healthcheck |
| `api_contract.rs` | 4 | Fixtures contrato |

---

## 6. Endpoints Oracle (MVP Fase 2)

| Método | Ruta | Auth |
|--------|------|------|
| `GET` | `/health` | No |
| `POST` | `/internal/v1/authorize` | X-API-KEY + allowlist + rate limit |
| `POST` | `/internal/v1/hold/release` | Idem |

---

## 7. Desviaciones documentadas

| Ítem | Plan original | Estado actual | Impacto |
|------|---------------|---------------|---------|
| Workspace incluye `oracle/` | Oracle fuera del workspace pasarela | Monorepo unificado con `oracle-client` | Bajo — frontera lógica preservada |
| `hold/consume` HTTP | Gate menciona consume | Endpoint en Fase 4; persistencia lista | Ninguno para MVP Oracle |
| Despliegue Docker | Dockerfile en entregables | Eliminado — binarios nativos (`cargo run`) | Ninguno — acordado con stakeholder |

---

## 8. Autorización

| Rol | Acción | Fecha |
|-----|--------|-------|
| Verificación técnica (dev) | Gate 2.11 — 87 tests verdes | 2026-07-25 |
| Stakeholder / Producto | **Confirmado — avanzar a Fase 3 / Fase 4** | 2026-07-25 |

**Próximo paso autorizado:** Fase 3 (programa Anchor `payment-settlement`) y/o Fase 4 (`crates/api-gateway/` + settlement adapters).

---

## Referencias

- [Plan-de-Implementacion.md §6](./Plan-de-Implementacion.md#6-fase-2--oracle-de-autorización-servicio-independiente)
- [API-v1.md](../oracle/docs/API-v1.md)
- [Acta-Cierre-Fase-1.md](./Acta-Cierre-Fase-1.md)
- [Acta-Cierre-Fase-4.md](./Acta-Cierre-Fase-4.md) — Gate Fase 4 (2026-07-26)
- [Acta-Cierre-Fase-0.md](./Acta-Cierre-Fase-0.md)
