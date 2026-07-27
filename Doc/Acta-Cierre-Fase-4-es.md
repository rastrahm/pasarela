# Acta de Cierre — Fase 4 (API Gateway y orquestación)

> Gate **4.14** · Verificación técnica y confirmación stakeholder  
> Fecha: **2026-07-26**  
> Proyecto: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaración

Se declara **cerrada la Fase 4 — API Gateway y orquestación**, con el backend principal (`crates/api-gateway/`) operativo: checkout multi-rail, consulta de transacciones, idempotencia (D9), auth por API key de comercio (D12), persistencia PostgreSQL opcional, liberación de hold en Oracle ante fallo de settlement, y suite E2E con mock Oracle + mock rieles.

Queda autorizado avanzar hacia **Fase 5 — Frontend (Dashboard & Checkout)**.

---

## 2. Entregables verificados

| Entregable | Ubicación | Estado |
|------------|-----------|--------|
| Settlement adapters (3 rieles) | `crates/settlement-adapters/` | ✅ |
| API Gateway Axum | `crates/api-gateway/` | ✅ |
| Orquestador checkout UC-01 | `crates/api-gateway/src/services/checkout.rs` | ✅ |
| Rail Switcher + Oracle integrados | `services/rails.rs`, `oracle-client` | ✅ |
| Consulta transacción UC-09 | `GET /api/v1/transactions/{id}` | ✅ |
| Mapeo errores HTTP §6.2 | `crates/api-gateway/src/error.rs` | ✅ |
| Idempotencia D9 | `services/idempotency.rs` | ✅ |
| Auth comercio D12 | `services/auth.rs`, `services/merchant.rs` | ✅ |
| Persistencia Gateway | `migrations/001_init.sql`, `src/persistence/` | ✅ |
| Liberación hold UC-04 | `release_hold_on_settlement_failure` + Oracle `/hold/release` | ✅ |
| Tests E2E gate | `tests/e2e_integration.rs` | ✅ |
| Variables de entorno | `crates/api-gateway/.env.example` | ✅ |
| Documentación operativa | `crates/api-gateway/README.md` (curl / Postman) | ✅ |

---

## 3. Checklist de pasos (Plan §8)

| # | Paso | Resultado |
|---|------|-----------|
| 4.1 | Crate `settlement-adapters/` | ✅ |
| 4.2 | Adapter `TraditionalBank` (ISO 20022 / ACH simulado) | ✅ |
| 4.3 | Adapter `BinanceCex` (API simulada + spread buffer) | ✅ |
| 4.4 | Adapter `SolanaWallet` (`solana-client`, commitment `finalized`) | ✅ |
| 4.5 | Crate `api-gateway/` (Axum, config, routes) | ✅ |
| 4.6 | `POST /api/v1/checkout` — orquestador completo | ✅ |
| 4.7 | Integración `rail-switcher` + `oracle-client` | ✅ |
| 4.8 | `GET /api/v1/transactions/{id}` (UC-09) | ✅ |
| 4.9 | Mapeo errores HTTP (200, 402, 422, 401, 503, 500) | ✅ |
| 4.10 | Idempotencia `Idempotency-Key` (D9) | ✅ |
| 4.11 | Auth API key comercio (D12) | ✅ |
| 4.12 | Persistencia Gateway (TRANSACTION, SETTLEMENT, AUDIT, MERCHANT) | ✅ |
| 4.13 | Liberar hold si settlement falla | ✅ |
| 4.14 | Tests integración Gateway + mock Oracle + mock rieles | ✅ |

---

## 4. Criterios de aceptación (gate)

| Criterio | Verificación | Resultado |
|----------|--------------|-----------|
| Checkout completo vía curl/Postman | `crates/api-gateway/README.md` + `gate_full_checkout_flow_settled_and_queryable` | ✅ |
| Tres rieles liquidan con proof distinto | `gate_three_rails_settle_with_distinct_proofs`, `three_rails_return_distinct_settlement_proofs` | ✅ |
| Fallback de riel operativo | `gate_rail_fallback_selects_next_viable_rail`, `fallback_selects_next_rail_and_authorizes_with_it` | ✅ |
| Hold liberado si settlement falla | `gate_hold_released_when_settlement_fails`, `checkout_releases_hold_when_settlement_fails` | ✅ |
| `cargo test` + integración verdes | 76 tests `api-gateway` + 32 `settlement-adapters` | ✅ |
| Confirmación stakeholder | Gate formal | ✅ 2026-07-26 |

---

## 5. Verificación técnica ejecutada

```bash
# Gateway (mock Oracle + mock rieles — sin PostgreSQL obligatorio)
cargo test -p api-gateway

# Gate E2E Fase 4
cargo test -p api-gateway --test e2e_integration

# Settlement adapters (Fase 4.1–4.4)
cargo test -p settlement-adapters
```

| Componente | Tests | Resultado |
|------------|-------|-----------|
| `api-gateway` (unitarios) | 39 | ✅ |
| `api-gateway` (integración) | 37 | ✅ |
| `settlement-adapters` | 32 | ✅ |
| **Total Fase 4** | **108** | ✅ |

### Suites de integración Gateway

| Archivo | Tests | Área |
|---------|-------|------|
| `e2e_integration.rs` | 9 | Gate Fase 4 — flujo completo |
| `checkout_integration.rs` | 6 | Checkout + auth + consulta |
| `rail_oracle_integration.rs` | 5 | Rail Switcher + Oracle |
| `error_mapping_integration.rs` | 8 | Códigos HTTP §6.2 |
| `hold_release_integration.rs` | 3 | Liberación hold UC-04 |
| `idempotency_integration.rs` | 3 | Idempotency-Key D9 |
| `http_integration.rs` | 3 | Health + transacciones |

---

## 6. Endpoints Gateway (MVP Fase 4)

| Método | Ruta | Auth |
|--------|------|------|
| `GET` | `/health` | No |
| `POST` | `/api/v1/checkout` | `Authorization: Bearer sk_*` + `Idempotency-Key` |
| `GET` | `/api/v1/transactions/{id}` | `Authorization: Bearer sk_*` |

### Flujo validado

```text
POST /api/v1/checkout
  → Auth comercio (D12)
  → Idempotencia (D9)
  → Rail Switcher (+ fallback D3)
  → Oracle POST /internal/v1/authorize
  → Settlement Engine (riel activo)
  → Persistencia (memoria o PostgreSQL)
  → 200 { transaction_id, status, settlement_proof }
     ó error mapeado (401/402/422/503/500)
  → Si settlement falla: POST /internal/v1/hold/release
```

---

## 7. Desviaciones documentadas

| Ítem | Plan original | Estado actual | Impacto |
|------|---------------|---------------|---------|
| `POST /hold/consume` | Mencionado en contrato Oracle v1.1 | Pendiente — no requerido para MVP Gateway | Ninguno — release cubre rollback |
| Persistencia PostgreSQL | Obligatoria en producción | Opcional: in-memory sin `DATABASE_URL`; Postgres con migraciones si está definida | Bajo — tests usan memoria |
| Cross-service E2E | Gateway + Oracle real | Cubierto con mock HTTP; Oracle real en Fase 6 | Ninguno para gate Fase 4 |
| Despliegue Docker | Entregables Fase 6 | Binarios nativos (`cargo run`) | Ninguno — acordado previamente |

---

## 8. Autorización

| Rol | Acción | Fecha |
|-----|--------|-------|
| Verificación técnica (dev) | Gate 4.14 — 108 tests verdes | 2026-07-26 |
| Stakeholder / Producto | **Confirmado — avanzar a Fase 5** | 2026-07-26 |

**Próximo paso autorizado:** Fase 5 — Frontend (`frontend/` con Vite + React + Tailwind, conexión al Gateway).

---

## Referencias

- [Plan-de-Implementacion.md §8](./Plan-de-Implementacion.md#8-fase-4--api-gateway-y-orquestación)
- [Arquitectura.md §6.2](./Arquitectura.md#62-api-gateway-y-orquestador-fase-4)
- [api-gateway/README.md](../crates/api-gateway/README.md)
- [settlement-adapters/README.md](../crates/settlement-adapters/README.md)
- [Acta-Cierre-Fase-2.md](./Acta-Cierre-Fase-2.md)
- [Acta-Cierre-Fase-1.md](./Acta-Cierre-Fase-1.md)
