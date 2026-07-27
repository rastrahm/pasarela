# Technical Debt Closure — Phase 6.8

> **Date:** 2026-07-26  
> **Scope:** P0/P1 items blocking or actionable before Phase 6 gate  
> **Living register:** [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)

---

## 1. Summary

| Priority | Before 6.8 | After 6.8 | Action |
|----------|------------|-----------|--------|
| **P0** | 4 open / mitigated | **0 blockers** | Closed or mitigated with evidence |
| **P1** | 6 pending | 3 resolved/mitigated | Fixtures + mocks; rest deferred post-gate |

**6.8 criterion:** no P0 item remains without documented resolution or mitigation evidence.

---

## 2. P0 — Final status

| ID | Status | Evidence |
|----|--------|----------|
| DT-P0-01 | **Resolved** | `cross_service_integration.rs` + CI with Postgres 16 |
| DT-P0-02 | **Resolved** | `.github/workflows/ci.yml` + [CI-en.md](./CI-en.md) |
| DT-P0-03 | **Resolved** | [Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md) + `scripts/check-env.sh` |
| DT-P0-04 | **Mitigated** | Canonical table runbook §4 + `check-env.sh`; code validation → Phase 7 |

---

## 3. P1 — Actions taken in 6.8

### DT-P1-01 — Divergent JSON fixtures ✅ Resolved

| Deliverable | Description |
|-------------|-------------|
| `scripts/fixtures/` | 6 checkout payloads + [README](../scripts/fixtures/README.md) |
| `crates/api-gateway/tests/common/fixtures.rs` | Rust loader |
| `crates/api-gateway/tests/contract_fixtures.rs` | Deserialization → `CheckoutRequest` |
| `frontend/src/schemas/gateway.test.ts` | Zod test vs `checkout-bank.json` |
| `oracle-client/tests/fixtures/authorize_request.json` | `expiry_year: "2030"` aligned |

Canonical convention: **`expiry_year` 4 digits (`"2030"`)** in public Gateway/frontend API.

### DT-P1-03 — Duplicate Oracle mocks 🔄 Partial

| File | Change |
|------|--------|
| `checkout_integration.rs` | Uses `spawn_mock_oracle` + `build_default_test_app` |
| `idempotency_integration.rs` | Uses `OracleCapture.authorize_calls` from shared mock |

**Pending post-gate:** `rail_oracle_integration.rs`, `error_mapping_integration.rs`, `hold_release_integration.rs` (specialized mocks per HTTP scenario).

### DT-P1-05 — Postgres password typo ✅ (prior to 6.2)

### DT-P1-06 — Playwright Node 18 ✅ Mitigated (`@playwright/test@1.59.1`)

---

## 4. P1 — Deferred (does not block gate)

| ID | Reason |
|----|--------|
| DT-P1-02 | Manual Zod mirror schemas — accepted MVP; OpenAPI in Phase 7 |
| DT-P1-04 | Monolithic `checkout.rs` — no refactor trigger in QA |
| DT-P1-03 (remainder) | Specialized mocks; consolidate when Oracle contract evolves |

---

## 5. P2 / P3 — No change

Remain registered in [Deuda-Tecnica-en.md §4–6](./Deuda-Tecnica-en.md) for post-gate or Phase 7.

---

## 6. Verification

```bash
# Contract fixtures
cargo test -p api-gateway --test contract_fixtures

# Consolidated mocks
cargo test -p api-gateway --test checkout_integration --test idempotency_integration

# Frontend ↔ canonical fixture
cd frontend && pnpm test:run src/schemas/gateway.test.ts

# Local environment
./scripts/check-env.sh
```

---

## 7. Phase 6 gate — pending (outside 6.8)

- [x] Playwright E2E green on 3 rails with real stack — local 2026-07-26 ([Pruebas-en.md](./Pruebas-en.md))
- [ ] QA checklist signed ([Checklist-QA-Fase-6-en.md §16](./Checklist-QA-Fase-6-en.md#16-approval-phase-6-gate))
- [ ] CI green on main branch (GitHub push)
- [ ] Explicit confirmation for Phase 7

---

## References

- [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening)
- [Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md)
- [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)
