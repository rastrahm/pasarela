# Deuda técnica — Pasarela Multi-Rail

> Registro vivo de deuda identificada antes y durante **Fase 6 — Integración, QA y hardening**.  
> Criterio: resolver ítems **P0/P1** cuando la Fase 6 los exponga como bloqueantes; el resto queda para post-gate o Fase 7.

**Última revisión:** 2026-07-26  
**Relacionado:** [Plan-de-Implementacion.md §6.8](./Plan-de-Implementacion.md#108-resolver-deuda-técnica-crítica)

---

## 1. Resumen ejecutivo

| Prioridad | Cantidad | Criterio de resolución |
|-----------|----------|------------------------|
| **P0** | 4 | Bloquea gate Fase 6 o E2E estable |
| **P1** | 6 | Fragilidad de contrato / mantenibilidad alta |
| **P2** | 6 | Limpieza; no bloquea MVP |
| **P3** | 5 | Post-MVP o decisión arquitectónica intencional |

**Decisión 2026-07-26:** posponer refactor estructural hasta completar pruebas de Fase 6 (Playwright, cross-service, checklist UC). Las necesidades reales de refactor se priorizarán según fallos observados en QA.

---

## 2. P0 — Bloqueantes para integración / gate Fase 6

### DT-P0-01 — Tests cross-service Gateway + Oracle real ausentes en `api-gateway`

| Campo | Valor |
|-------|-------|
| **Estado** | **Mitigado** (2026-07-26) |
| **Evidencia** | `crates/api-gateway/tests/cross_service_integration.rs` — 7 tests con `spawn_oracle_server()` de `oracle/src/test_support.rs` |
| **Requisito** | PostgreSQL en `ORACLE_DATABASE_URL` (default `postgres://postgres:postgres@localhost:5432/oracle_test`) |
| **Ejecución** | `cargo test -p api-gateway --test cross_service_integration -- --test-threads=1` |
| **Nota** | Sin PostgreSQL los tests se omiten con mensaje `SKIP` (no fallan) |

### DT-P0-02 — CI unificado ausente (Rust + frontend + E2E)

| Campo | Valor |
|-------|-------|
| **Estado** | **Resuelto** (2026-07-26) |
| **Evidencia** | `.github/workflows/ci.yml` — jobs `rust`, `frontend`, `e2e` |
| **Anchor** | `.github/workflows/programs-anchor-test.yml` (path-filtered) |
| **Doc** | [Doc/CI.md](./CI.md) |

### DT-P0-03 — Runbook de stack local disperso

| Campo | Valor |
|-------|-------|
| **Estado** | **Resuelto** (2026-07-26) |
| **Evidencia** | [Doc/Runbook-Desarrollo.md](./Runbook-Desarrollo.md) — orden de arranque, matriz por riel, troubleshooting |
| **Script** | [`scripts/check-env.sh`](../scripts/check-env.sh) |
| **Servicios** | binance-sim → antifraud → oracle → gateway → frontend |

### DT-P0-04 — Variables de entorno desincronizadas entre servicios

| Campo | Valor |
|-------|-------|
| **Estado** | **Mitigado** (2026-07-26) |
| **Evidencia** | Tabla canónica en [Runbook-Desarrollo.md §4](./Runbook-Desarrollo.md#4-tabla-canónica-de-variables); validación `./scripts/check-env.sh` |
| **Impacto** | Fallos en runtime difíciles de diagnosticar |
| **Pendiente** | Validación al arranque en código (opcional, post-gate) |

---

## 3. P1 — Fragilidad de contrato y tests

### DT-P1-01 — Fixtures JSON divergentes (Gateway / Oracle / frontend)

| Campo | Valor |
|-------|-------|
| **Estado** | Pendiente |
| **Archivos** | `crates/oracle-client/tests/fixtures/` (`expiry_year: "30"`), `crates/api-gateway/tests/common/http.rs` (`"2030"`), `frontend/src/schemas/gateway.ts` |
| **Impacto** | Falsos negativos en E2E y cross-service |
| **Acción** | Directorio canónico `crates/oracle-client/tests/fixtures/` referenciado por Gateway y frontend |

### DT-P1-02 — Schemas frontend espejo manual del Gateway

| Campo | Valor |
|-------|-------|
| **Estado** | Aceptado (MVP); revisar post-Fase 6 |
| **Archivos** | `frontend/src/schemas/gateway.ts` ↔ `crates/api-gateway/src/routes/dto.rs`, `error.rs` |
| **Impacto** | Cambio en Gateway puede romper checkout sin fallo obvio |
| **Acción mínima** | Test de contrato JSON Gateway ↔ Zod |
| **Acción media** | OpenAPI + generación de tipos TS |

### DT-P1-03 — Mocks Oracle duplicados en tests del Gateway

| Campo | Valor |
|-------|-------|
| **Estado** | Pendiente |
| **Archivos** | `tests/common/mock_oracle.rs` (canónico) + inline en `rail_oracle_integration.rs`, `error_mapping_integration.rs`, `hold_release_integration.rs`, `checkout_integration.rs`, `idempotency_integration.rs` |
| **Impacto** | Mantenimiento costoso al evolucionar contrato Oracle |
| **Acción** | Consolidar en `tests/common/mock_oracle.rs` con opciones configurables |

### DT-P1-04 — `checkout.rs` monolítico (505 líneas)

| Campo | Valor |
|-------|-------|
| **Estado** | Pendiente |
| **Archivo** | `crates/api-gateway/src/services/checkout.rs` |
| **Impacto** | Dificulta extender casos edge (fallback, release hold) en Fase 6 |
| **Acción** | Submódulos: `oracle_bridge.rs`, `persist.rs`, `settlement.rs` |
| **Trigger refactor** | Cuando Fase 6 añada casos que requieran tocar >3 responsabilidades distintas |

### DT-P1-05 — Typo password Postgres en tests Oracle

| Campo | Valor |
|-------|-------|
| **Estado** | **Resuelto** (2026-07-26) |
| **Archivo** | `oracle/src/test_support.rs` — default `postgres://postgres:postgres@...` |
| **Acción** | Corregido al centralizar helpers en `test_support` |

### DT-P1-06 — Playwright requiere versión fijada en Node 18

| Campo | Valor |
|-------|-------|
| **Estado** | Mitigado |
| **Evidencia** | `@playwright/test@1.62+` exige Node 20; entorno dev en Node 18.20.8 |
| **Acción** | Fijar `@playwright/test@1.59.1` en `tests/e2e/package.json`; migrar a Node 20+ cuando el toolchain lo permita |

---

## 4. P2 — Mantenibilidad y limpieza

### DT-P2-01 — Traits de dominio sin implementación

| Campo | Valor |
|-------|-------|
| **Archivo** | `crates/domain/src/traits.rs` — `PaymentProcessor`, `LiquidityEngine` |
| **Acción** | Implementar facade o eliminar si no hay plan de uso |

### DT-P2-02 — `.expect()` en código de producción (Solana)

| Campo | Valor |
|-------|-------|
| **Archivo** | `crates/settlement-adapters/src/solana/instruction.rs:56` |
| **Acción** | Reemplazar por `map_err` → `RailError` |

### DT-P2-03 — Luhn duplicado en Oracle (validación vs redacción logs)

| Campo | Valor |
|-------|-------|
| **Archivos** | `oracle/src/validation/mod.rs`, `oracle/src/logging/redact.rs` |
| **Acción** | Reutilizar helper de validación en redact |

### DT-P2-04 — Cliente HTTP Binance duplicado

| Campo | Valor |
|-------|-------|
| **Archivos** | `oracle/src/rail_adapters/http_binance.rs`, `crates/settlement-adapters/src/binance/client.rs` |
| **Acción** | Evaluar crate interno `binance-sim-client` post-gate |

### DT-P2-05 — Cliente RPC Solana duplicado

| Campo | Valor |
|-------|-------|
| **Archivos** | `oracle/src/rail_adapters/rpc_solana.rs`, `crates/settlement-adapters/src/solana/client.rs` |
| **Acción** | Compartir utilidades RPC post-gate |

### DT-P2-06 — Tests inline en `rail-switcher`

| Campo | Valor |
|-------|-------|
| **Archivo** | `crates/rail-switcher/src/switcher.rs` (~416 líneas) |
| **Acción** | Mover tests a `crates/rail-switcher/tests/` |

---

## 5. Configuración y entorno

### Variables que deben coincidir

| Variable | Servicios | Archivos `.env.example` |
|----------|-----------|-------------------------|
| `ORACLE_API_KEY` | Gateway, Oracle | `crates/api-gateway/`, `oracle/` |
| `GATEWAY_TEST_API_KEY` ↔ `VITE_GATEWAY_API_KEY` | Gateway, Frontend | `crates/api-gateway/`, `frontend/` |
| `BINANCE_CEX_API_KEY` | Oracle, binance-sim, settlement-adapters | 3 archivos |
| `ANTIFRAUD_API_KEY` | Oracle, antifraud | 2 archivos |
| `BINANCE_CEX_BASE_URL` | Oracle, settlement-adapters | comentado en Gateway |
| `BINANCE_SPREAD_BUFFER_PCT` | Oracle, settlement-adapters | sincronización manual |
| `SOLANA_RPC_URL` | Oracle, settlement-adapters | comentado en Gateway |

### Bases de datos Postgres distintas (intencional en MVP)

| Servicio | URL default |
|----------|-------------|
| Gateway | `postgres://pasarela:pasarela@127.0.0.1:5432/pasarela_gateway` |
| Oracle | `postgres://postgres:postgres@localhost:5432/oracle` |

> Gateway puede operar in-memory sin `DATABASE_URL` (solo dev/tests).

---

## 6. P3 — Post-MVP / arquitectura intencional (no refactorizar ahora)

| ID | Ítem | Motivo para posponer |
|----|------|----------------------|
| DT-P3-01 | Unificar `FundingType` en un solo crate | Rompe frontera Oracle ↔ domain (Arquitectura §4) |
| DT-P3-02 | Eliminar Luhn del frontend | Empeora UX; Oracle sigue siendo autoridad |
| DT-P3-03 | Implementar traits `PaymentProcessor` | Cambio arquitectónico, no deuda funcional |
| DT-P3-04 | Pantalla UC-09 consulta transacción | Feature nueva; `fetchTransaction` ya existe en API |
| DT-P3-05 | Dashboard admin | Fuera de alcance MVP (Acta Fase 5 §7) |

### SEC-6.4-01 — IDOR consulta transacciones

| Campo | Valor |
|-------|-------|
| **Estado** | **Resuelto** (2026-07-26) |
| **Fix** | Filtro `merchant_id` en `GET /api/v1/transactions/:id` |
| **Test** | `security_integration.rs` |

### SEC-6.4-02 — PAN en fingerprint idempotencia

| Campo | Valor |
|-------|-------|
| **Estado** | **Resuelto** (2026-07-26) |
| **Fix** | `request_fingerprint` sin PAN/CVV; SHA-256 del PAN |
| **Test** | `idempotency.rs` unit + `security_integration.rs` |

### SEC-C-03 — API key comercio en bundle frontend

| Campo | Valor |
|-------|-------|
| **Estado** | Aceptado MVP demo |
| **Evidencia** | `frontend/src/config/env.ts` — `VITE_GATEWAY_API_KEY` |
| **Plan** | Fase 7 BFF; ver [Revision-Seguridad-Fase-6.md](./Revision-Seguridad-Fase-6.md) |

---

## 7. Deuda documentada en actas y arquitectura

| Fuente | Ítem |
|--------|------|
| [Acta-Cierre-Fase-4.md §7](./Acta-Cierre-Fase-4.md) | Cross-service real; `POST /hold/consume` pendiente |
| [Acta-Cierre-Fase-5.md §7](./Acta-Cierre-Fase-5.md) | Playwright Fase 6; Node 18 + happy-dom; sin dashboard |
| [Arquitectura.md §12](./Arquitectura.md) | mTLS Fase 7/8; 3DS post-MVP; HSM tokenización post-MVP |
| `oracle/docs/API-v1.md` | `POST /internal/v1/hold/consume` planificado v1.1 |
| `oracle/src/validation/mod.rs` | `hash_pan` determinístico demo — reemplazar HMAC-SHA256 en producción |
| `programs/payment-settlement/` | Fuera workspace Cargo; toolchain Anchor separada |

---

## 8. Duplicación de tipos (aceptada en MVP)

| Tipo | Ubicaciones | Conversión |
|------|-------------|------------|
| `FundingType` | `domain`, `oracle-client`, `oracle/funds`, `frontend/schemas` | `to_oracle_funding_type()` en checkout |
| `CardPayload` | `domain`, `oracle-client`, Gateway DTO, frontend Zod | `to_oracle_card()` en checkout |
| `FundStatus` | `domain`, `oracle/funds` | From/Into manual |

> La duplicación refleja fronteras wire vs dominio. Refactor solo si Fase 6 demuestra costo operativo alto.

---

## 9. Matriz de resolución vs Fase 6

| Paso Fase 6 | Deuda que puede activarse |
|-------------|---------------------------|
| **6.1** Playwright E2E | DT-P1-01, DT-P0-03, DT-P0-04 |
| **6.2** Cross-service | DT-P0-01, DT-P1-03, DT-P1-05 |
| **6.5** Rendimiento | PERF-R-01 async Solana UX | Post-MVP |
| **6.6** CI | DT-P0-02 |
| **6.7** Runbook | DT-P0-03, DT-P0-04 |
| **6.8** Deuda crítica | Revisar ítems P0 marcados como bloqueantes en esta tabla |

---

## 10. Registro de cambios

| Fecha | Cambio |
|-------|--------|
| 2026-07-26 | CI unificado 6.6 — `.github/workflows/ci.yml` + [Doc/CI.md](./CI.md) |
| 2026-07-26 | Revisión seguridad 6.4 — [Revision-Seguridad-Fase-6.md](./Revision-Seguridad-Fase-6.md); fixes IDOR + idempotency PCI |
| 2026-07-26 | DT-P0-01 mitigado — `cross_service_integration.rs` + `oracle/src/test_support.rs` |

---

## Referencias

- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Arquitectura.md §4](./Arquitectura.md#4-reglas-de-dependencia)
- [Acta-Cierre-Fase-5.md](./Acta-Cierre-Fase-5.md)
- [Acta-Cierre-Fase-4.md](./Acta-Cierre-Fase-4.md)
