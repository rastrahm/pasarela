# Phase Closure Record — Phase 4 (API Gateway and orchestration)

> Gate **4.14** · Technical verification and stakeholder confirmation  
> Date: **2026-07-26**  
> Project: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaration

**Phase 4 — API Gateway and orchestration** is declared **closed**, with the main backend (`crates/api-gateway/`) operational: multi-rail checkout, transaction lookup, idempotency (D9), merchant API key auth (D12), optional PostgreSQL persistence, Oracle hold release on settlement failure, and E2E suite with mock Oracle + mock rails.

Advancement toward **Phase 5 — Frontend (Dashboard & Checkout)** is authorized.

---

## 2. Verified deliverables

| Deliverable | Location | Status |
|-------------|----------|--------|
| Settlement adapters (3 rails) | `crates/settlement-adapters/` | ✅ |
| Axum API Gateway | `crates/api-gateway/` | ✅ |
| Checkout orchestrator UC-01 | `crates/api-gateway/src/services/checkout.rs` | ✅ |
| Integrated Rail Switcher + Oracle | `services/rails.rs`, `oracle-client` | ✅ |
| Transaction lookup UC-09 | `GET /api/v1/transactions/{id}` | ✅ |
| HTTP error mapping §6.2 | `crates/api-gateway/src/error.rs` | ✅ |
| Idempotency D9 | `services/idempotency.rs` | ✅ |
| Merchant auth D12 | `services/auth.rs`, `services/merchant.rs` | ✅ |
| Gateway persistence | `migrations/001_init.sql`, `src/persistence/` | ✅ |
| Hold release UC-04 | `release_hold_on_settlement_failure` + Oracle `/hold/release` | ✅ |
| Gate E2E tests | `tests/e2e_integration.rs` | ✅ |
| Environment variables | `crates/api-gateway/.env.example` | ✅ |
| Operational documentation | `crates/api-gateway/README.md` (curl / Postman) | ✅ |

---

## 3. Step checklist (Plan §8)

| # | Step | Result |
|---|------|--------|
| 4.1 | Crate `settlement-adapters/` | ✅ |
| 4.2 | `TraditionalBank` adapter (simulated ISO 20022 / ACH) | ✅ |
| 4.3 | `BinanceCex` adapter (simulated API + spread buffer) | ✅ |
| 4.4 | `SolanaWallet` adapter (`solana-client`, commitment `finalized`) | ✅ |
| 4.5 | Crate `api-gateway/` (Axum, config, routes) | ✅ |
| 4.6 | `POST /api/v1/checkout` — full orchestrator | ✅ |
| 4.7 | `rail-switcher` + `oracle-client` integration | ✅ |
| 4.8 | `GET /api/v1/transactions/{id}` (UC-09) | ✅ |
| 4.9 | HTTP error mapping (200, 402, 422, 401, 503, 500) | ✅ |
| 4.10 | `Idempotency-Key` idempotency (D9) | ✅ |
| 4.11 | Merchant API key auth (D12) | ✅ |
| 4.12 | Gateway persistence (TRANSACTION, SETTLEMENT, AUDIT, MERCHANT) | ✅ |
| 4.13 | Release hold if settlement fails | ✅ |
| 4.14 | Gateway integration tests + mock Oracle + mock rails | ✅ |

---

## 4. Acceptance criteria (gate)

| Criterion | Verification | Result |
|-----------|--------------|--------|
| Full checkout via curl/Postman | `crates/api-gateway/README.md` + `gate_full_checkout_flow_settled_and_queryable` | ✅ |
| Three rails settle with distinct proof | `gate_three_rails_settle_with_distinct_proofs`, `three_rails_return_distinct_settlement_proofs` | ✅ |
| Operational rail fallback | `gate_rail_fallback_selects_next_viable_rail`, `fallback_selects_next_rail_and_authorizes_with_it` | ✅ |
| Hold released if settlement fails | `gate_hold_released_when_settlement_fails`, `checkout_releases_hold_when_settlement_fails` | ✅ |
| Green `cargo test` + integration | 76 `api-gateway` tests + 32 `settlement-adapters` | ✅ |
| Stakeholder confirmation | Formal gate | ✅ 2026-07-26 |

---

## 5. Technical verification executed

```bash
# Gateway (mock Oracle + mock rails — PostgreSQL not required)
cargo test -p api-gateway

# Phase 4 E2E gate
cargo test -p api-gateway --test e2e_integration

# Settlement adapters (Phase 4.1–4.4)
cargo test -p settlement-adapters
```

| Component | Tests | Result |
|-----------|-------|--------|
| `api-gateway` (unit) | 39 | ✅ |
| `api-gateway` (integration) | 37 | ✅ |
| `settlement-adapters` | 32 | ✅ |
| **Phase 4 total** | **108** | ✅ |

### Gateway integration suites

| File | Tests | Area |
|------|-------|------|
| `e2e_integration.rs` | 9 | Phase 4 gate — full flow |
| `checkout_integration.rs` | 6 | Checkout + auth + lookup |
| `rail_oracle_integration.rs` | 5 | Rail Switcher + Oracle |
| `error_mapping_integration.rs` | 8 | HTTP codes §6.2 |
| `hold_release_integration.rs` | 3 | Hold release UC-04 |
| `idempotency_integration.rs` | 3 | Idempotency-Key D9 |
| `http_integration.rs` | 3 | Health + transactions |

---

## 6. Gateway endpoints (Phase 4 MVP)

| Method | Route | Auth |
|--------|-------|------|
| `GET` | `/health` | No |
| `POST` | `/api/v1/checkout` | `Authorization: Bearer sk_*` + `Idempotency-Key` |
| `GET` | `/api/v1/transactions/{id}` | `Authorization: Bearer sk_*` |

### Validated flow

```text
POST /api/v1/checkout
  → Merchant auth (D12)
  → Idempotency (D9)
  → Rail Switcher (+ fallback D3)
  → Oracle POST /internal/v1/authorize
  → Settlement Engine (active rail)
  → Persistence (memory or PostgreSQL)
  → 200 { transaction_id, status, settlement_proof }
     or mapped error (401/402/422/503/500)
  → If settlement fails: POST /internal/v1/hold/release
```

---

## 7. Documented deviations

| Item | Original plan | Current state | Impact |
|------|---------------|---------------|--------|
| `POST /hold/consume` | Mentioned in Oracle v1.1 contract | Pending — not required for Gateway MVP | None — release covers rollback |
| PostgreSQL persistence | Required in production | Optional: in-memory without `DATABASE_URL`; Postgres with migrations if defined | Low — tests use memory |
| Cross-service E2E | Gateway + real Oracle | Covered with HTTP mock; real Oracle in Phase 6 | None for Phase 4 gate |
| Docker deployment | Phase 6 deliverables | Native binaries (`cargo run`) | None — previously agreed |

---

## 8. Authorization

| Role | Action | Date |
|------|--------|------|
| Technical verification (dev) | Gate 4.14 — 108 green tests | 2026-07-26 |
| Stakeholder / Product | **Confirmed — advance to Phase 5** | 2026-07-26 |

**Authorized next step:** Phase 5 — Frontend (`frontend/` with Vite + React + Tailwind, connected to Gateway).

---

## References

- [Plan-de-Implementacion-en.md §8](./Plan-de-Implementacion-en.md#8-fase-4--api-gateway-y-orquestación)
- [Arquitectura-en.md §6.2](./Arquitectura-en.md#62-api-gateway-y-orquestador-fase-4)
- [api-gateway/README.md](../crates/api-gateway/README.md)
- [settlement-adapters/README.md](../crates/settlement-adapters/README.md)
- [Acta-Cierre-Fase-2-en.md](./Acta-Cierre-Fase-2-en.md)
- [Acta-Cierre-Fase-1-en.md](./Acta-Cierre-Fase-1-en.md)
