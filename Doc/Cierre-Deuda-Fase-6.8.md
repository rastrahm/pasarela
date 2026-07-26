# Cierre deuda técnica — Fase 6.8

> **Fecha:** 2026-07-26  
> **Alcance:** ítems P0/P1 bloqueantes o accionables antes del gate Fase 6  
> **Registro vivo:** [Deuda-Tecnica.md](./Deuda-Tecnica.md)

---

## 1. Resumen

| Prioridad | Antes 6.8 | Tras 6.8 | Acción |
|-----------|-----------|----------|--------|
| **P0** | 4 abiertos / mitigados | **0 bloqueantes** | Cerrados o mitigados con evidencia |
| **P1** | 6 pendientes | 3 resueltos/mitigados | Fixtures + mocks; resto pospuesto post-gate |

**Criterio 6.8:** ningún ítem P0 queda sin evidencia de resolución o mitigación documentada.

---

## 2. P0 — Estado final

| ID | Estado | Evidencia |
|----|--------|-----------|
| DT-P0-01 | **Resuelto** | `cross_service_integration.rs` + CI con Postgres 16 |
| DT-P0-02 | **Resuelto** | `.github/workflows/ci.yml` + [CI.md](./CI.md) |
| DT-P0-03 | **Resuelto** | [Runbook-Desarrollo.md](./Runbook-Desarrollo.md) + `scripts/check-env.sh` |
| DT-P0-04 | **Mitigado** | Tabla canónica runbook §4 + `check-env.sh`; validación en código → Fase 7 |

---

## 3. P1 — Acciones realizadas en 6.8

### DT-P1-01 — Fixtures JSON divergentes ✅ Resuelto

| Entregable | Descripción |
|------------|-------------|
| `scripts/fixtures/` | 6 payloads checkout + [README](../scripts/fixtures/README.md) |
| `crates/api-gateway/tests/common/fixtures.rs` | Loader Rust |
| `crates/api-gateway/tests/contract_fixtures.rs` | Deserialización → `CheckoutRequest` |
| `frontend/src/schemas/gateway.test.ts` | Test Zod vs `checkout-bank.json` |
| `oracle-client/tests/fixtures/authorize_request.json` | `expiry_year: "2030"` alineado |

Convención canónica: **`expiry_year` de 4 dígitos (`"2030"`)** en API pública Gateway/frontend.

### DT-P1-03 — Mocks Oracle duplicados 🔄 Parcial

| Archivo | Cambio |
|---------|--------|
| `checkout_integration.rs` | Usa `spawn_mock_oracle` + `build_default_test_app` |
| `idempotency_integration.rs` | Usa `OracleCapture.authorize_calls` del mock común |

**Pendiente post-gate:** `rail_oracle_integration.rs`, `error_mapping_integration.rs`, `hold_release_integration.rs` (mocks especializados por escenario HTTP).

### DT-P1-05 — Typo Postgres ✅ (previo 6.2)

### DT-P1-06 — Playwright Node 18 ✅ Mitigado (`@playwright/test@1.59.1`)

---

## 4. P1 — Pospuesto (no bloquea gate)

| ID | Motivo |
|----|--------|
| DT-P1-02 | Schemas Zod espejo manual — aceptado MVP; OpenAPI en Fase 7 |
| DT-P1-04 | `checkout.rs` monolítico — sin trigger de refactor en QA |
| DT-P1-03 (resto) | Mocks especializados; consolidar cuando evolucione contrato Oracle |

---

## 5. P2 / P3 — Sin cambio

Quedan registrados en [Deuda-Tecnica.md §4–6](./Deuda-Tecnica.md) para post-gate o Fase 7.

---

## 6. Verificación

```bash
# Contrato fixtures
cargo test -p api-gateway --test contract_fixtures

# Mocks consolidados
cargo test -p api-gateway --test checkout_integration --test idempotency_integration

# Frontend ↔ fixture canónico
cd frontend && pnpm test:run src/schemas/gateway.test.ts

# Entorno local
./scripts/check-env.sh
```

---

## 7. Gate Fase 6 — pendiente (fuera de 6.8)

- [ ] Playwright E2E verde en 3 rieles con stack real
- [ ] Checklist QA firmado ([Checklist-QA-Fase-6.md §16](./Checklist-QA-Fase-6.md#16-aprobación-gate-fase-6))
- [ ] CI verde en rama principal (push GitHub)
- [ ] Confirmación explícita para Fase 7

---

## Referencias

- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Runbook-Desarrollo.md](./Runbook-Desarrollo.md)
- [Deuda-Tecnica.md](./Deuda-Tecnica.md)
