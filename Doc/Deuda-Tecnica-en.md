# Technical Debt — Pasarela Multi-Rail

> Living register of debt identified before and during **Phase 6 — Integration, QA, and hardening**.  
> Criterion: resolve **P0/P1** items when Phase 6 exposes them as blockers; the rest remains for post-gate or Phase 7.

**Last review:** 2026-07-26  
**Related:** [Plan-de-Implementacion-en.md §6.8](./Plan-de-Implementacion-en.md#108-resolver-deuda-técnica-crítica)

---

## 1. Executive summary

| Priority | Count | Resolution criterion |
|----------|-------|----------------------|
| **P0** | 0 blockers | Resolved or mitigated — see [Cierre-Deuda-Fase-6.8-en.md](./Cierre-Deuda-Fase-6.8-en.md) |
| **P1** | 6 | Contract fragility / high maintainability |
| **P2** | 6 | Cleanup; does not block MVP |
| **P3** | 5 | Post-MVP or intentional architectural decision |

**Decision 2026-07-26:** defer structural refactor until Phase 6 tests complete (Playwright, cross-service, UC checklist). Real refactor needs will be prioritized based on failures observed in QA.

---

## 2. P0 — Blockers for integration / Phase 6 gate

### DT-P0-01 — Cross-service Gateway + real Oracle tests missing in `api-gateway`

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Evidence** | `crates/api-gateway/tests/cross_service_integration.rs` — 7 tests; CI `rust` job with Postgres 16 |
| **Requirement** | PostgreSQL in `ORACLE_DATABASE_URL` (default `postgres://postgres:postgres@localhost:5432/oracle_test`) |
| **Execution** | `cargo test -p api-gateway --test cross_service_integration -- --test-threads=1` |
| **Note** | Without PostgreSQL tests skip with `SKIP` message (do not fail) |

### DT-P0-02 — Unified CI missing (Rust + frontend + E2E)

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Evidence** | `.github/workflows/ci.yml` — jobs `rust`, `frontend`, `e2e` |
| **Anchor** | `.github/workflows/programs-anchor-test.yml` (path-filtered) |
| **Doc** | [Doc/CI-en.md](./CI-en.md) |

### DT-P0-03 — Dispersed local stack runbook

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Evidence** | [Doc/Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md) — startup order, rail matrix, troubleshooting |
| **Script** | [`scripts/check-env.sh`](../scripts/check-env.sh) |
| **Services** | binance-sim → antifraud → oracle → gateway → frontend |

### DT-P0-04 — Environment variables out of sync between services

| Field | Value |
|-------|-------|
| **Status** | **Mitigated** (2026-07-26) |
| **Evidence** | Canonical table in [Runbook-Desarrollo-en.md §4](./Runbook-Desarrollo-en.md#4-canonical-variable-table); validation `./scripts/check-env.sh` |
| **Impact** | Runtime failures hard to diagnose |
| **Pending** | Startup validation in code (optional, post-gate) |

---

## 3. P1 — Contract fragility and tests

### DT-P1-01 — Divergent JSON fixtures (Gateway / Oracle / frontend)

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Evidence** | `scripts/fixtures/` + `contract_fixtures.rs` + Zod test in `gateway.test.ts` |
| **Convention** | `expiry_year: "2030"` (4 digits) in public API |
| **Doc** | [scripts/fixtures/README.md](../scripts/fixtures/README.md) |

### DT-P1-02 — Frontend schemas manually mirror Gateway

| Field | Value |
|-------|-------|
| **Status** | Accepted (MVP); review post-Phase 6 |
| **Files** | `frontend/src/schemas/gateway.ts` ↔ `crates/api-gateway/src/routes/dto.rs`, `error.rs` |
| **Impact** | Gateway change can break checkout without obvious failure |
| **Minimum action** | JSON contract test Gateway ↔ Zod |
| **Medium action** | OpenAPI + TS type generation |

### DT-P1-03 — Duplicate Oracle mocks in Gateway tests

| Field | Value |
|-------|-------|
| **Status** | **Partial** (2026-07-26) |
| **Consolidated** | `checkout_integration.rs`, `idempotency_integration.rs` → `tests/common/mock_oracle.rs` |
| **Pending** | `rail_oracle_integration.rs`, `error_mapping_integration.rs`, `hold_release_integration.rs` |
| **Post-gate action** | Extend `MockOracleOpts` for custom HTTP errors |

### DT-P1-04 — Monolithic `checkout.rs` (505 lines)

| Field | Value |
|-------|-------|
| **Status** | Pending |
| **File** | `crates/api-gateway/src/services/checkout.rs` |
| **Impact** | Hard to extend edge cases (fallback, release hold) in Phase 6 |
| **Action** | Submodules: `oracle_bridge.rs`, `persist.rs`, `settlement.rs` |
| **Refactor trigger** | When Phase 6 adds cases requiring >3 distinct responsibilities |

### DT-P1-05 — Postgres password typo in Oracle tests

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **File** | `oracle/src/test_support.rs` — default `postgres://postgres:postgres@...` |
| **Action** | Fixed when centralizing helpers in `test_support` |

### DT-P1-06 — Playwright requires pinned version on Node 18

| Field | Value |
|-------|-------|
| **Status** | Mitigated |
| **Evidence** | `@playwright/test@1.62+` requires Node 20; dev environment on Node 18.20.8 |
| **Action** | Pin `@playwright/test@1.59.1` in `tests/e2e/package.json`; migrate to Node 20+ when toolchain allows |

---

## 4. P2 — Maintainability and cleanup

### DT-P2-01 — Domain traits without implementation

| Field | Value |
|-------|-------|
| **File** | `crates/domain/src/traits.rs` — `PaymentProcessor`, `LiquidityEngine` |
| **Action** | Implement facade or remove if no usage plan |

### DT-P2-02 — `.expect()` in production code (Solana)

| Field | Value |
|-------|-------|
| **File** | `crates/settlement-adapters/src/solana/instruction.rs:56` |
| **Action** | Replace with `map_err` → `RailError` |

### DT-P2-03 — Luhn duplicated in Oracle (validation vs log redaction)

| Field | Value |
|-------|-------|
| **Files** | `oracle/src/validation/mod.rs`, `oracle/src/logging/redact.rs` |
| **Action** | Reuse validation helper in redact |

### DT-P2-04 — Duplicate Binance HTTP client

| Field | Value |
|-------|-------|
| **Files** | `oracle/src/rail_adapters/http_binance.rs`, `crates/settlement-adapters/src/binance/client.rs` |
| **Action** | Evaluate internal `binance-sim-client` crate post-gate |

### DT-P2-05 — Duplicate Solana RPC client

| Field | Value |
|-------|-------|
| **Files** | `oracle/src/rail_adapters/rpc_solana.rs`, `crates/settlement-adapters/src/solana/client.rs` |
| **Action** | Share RPC utilities post-gate |

### DT-P2-06 — Inline tests in `rail-switcher`

| Field | Value |
|-------|-------|
| **File** | `crates/rail-switcher/src/switcher.rs` (~416 lines) |
| **Action** | Move tests to `crates/rail-switcher/tests/` |

---

## 5. Configuration and environment

### Variables that must match

| Variable | Services | `.env.example` files |
|----------|----------|----------------------|
| `ORACLE_API_KEY` | Gateway, Oracle | `crates/api-gateway/`, `oracle/` |
| `GATEWAY_TEST_API_KEY` ↔ `VITE_GATEWAY_API_KEY` | Gateway, Frontend | `crates/api-gateway/`, `frontend/` |
| `BINANCE_CEX_API_KEY` | Oracle, binance-sim, settlement-adapters | 3 files |
| `ANTIFRAUD_API_KEY` | Oracle, antifraud | 2 files |
| `BINANCE_CEX_BASE_URL` | Oracle, settlement-adapters | commented in Gateway |
| `BINANCE_SPREAD_BUFFER_PCT` | Oracle, settlement-adapters | manual sync |
| `SOLANA_RPC_URL` | Oracle, settlement-adapters | commented in Gateway |

### Distinct Postgres databases (intentional in MVP)

| Service | Default URL |
|---------|-------------|
| Gateway | `postgres://pasarela:pasarela@127.0.0.1:5432/pasarela_gateway` |
| Oracle | `postgres://postgres:postgres@localhost:5432/oracle` |

> Gateway can operate in-memory without `DATABASE_URL` (dev/tests only).

---

## 6. P3 — Post-MVP / intentional architecture (do not refactor now)

| ID | Item | Reason to defer |
|----|------|-----------------|
| DT-P3-01 | Unify `FundingType` in single crate | Breaks Oracle ↔ domain boundary (Architecture §4) |
| DT-P3-02 | Remove Luhn from frontend | Worsens UX; Oracle remains authority |
| DT-P3-03 | Implement `PaymentProcessor` traits | Architectural change, not functional debt |
| DT-P3-04 | UC-09 transaction query screen | New feature; `fetchTransaction` already exists in API |
| DT-P3-05 | Admin dashboard | Out of MVP scope (Phase 5 Act §7) |

### SEC-6.4-01 — IDOR transaction lookup

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Fix** | `merchant_id` filter on `GET /api/v1/transactions/:id` |
| **Test** | `security_integration.rs` |

### SEC-6.4-02 — PAN in idempotency fingerprint

| Field | Value |
|-------|-------|
| **Status** | **Resolved** (2026-07-26) |
| **Fix** | `request_fingerprint` without PAN/CVV; SHA-256 of PAN |
| **Test** | `idempotency.rs` unit + `security_integration.rs` |

### SEC-C-03 — Merchant API key in frontend bundle

| Field | Value |
|-------|-------|
| **Status** | Accepted MVP demo |
| **Evidence** | `frontend/src/config/env.ts` — `VITE_GATEWAY_API_KEY` |
| **Plan** | Phase 7 BFF; see [Revision-Seguridad-Fase-6-en.md](./Revision-Seguridad-Fase-6-en.md) |

---

## 7. Debt documented in actas and architecture

| Source | Item |
|--------|------|
| [Acta-Cierre-Fase-4-en.md §7](./Acta-Cierre-Fase-4-en.md) | Real cross-service; pending `POST /hold/consume` |
| [Acta-Cierre-Fase-5-en.md §7](./Acta-Cierre-Fase-5-en.md) | Playwright Phase 6; Node 18 + happy-dom; no dashboard |
| [Arquitectura-en.md §12](./Arquitectura-en.md) | mTLS Phase 7/8; 3DS post-MVP; HSM tokenization post-MVP |
| `oracle/docs/API-v1.md` | `POST /internal/v1/hold/consume` planned v1.1 |
| `oracle/src/validation/mod.rs` | Deterministic demo `hash_pan` — replace with HMAC-SHA256 in production |
| `programs/payment-settlement/` | Outside Cargo workspace; separate Anchor toolchain |

---

## 8. Type duplication (accepted in MVP)

| Type | Locations | Conversion |
|------|-----------|------------|
| `FundingType` | `domain`, `oracle-client`, `oracle/funds`, `frontend/schemas` | `to_oracle_funding_type()` in checkout |
| `CardPayload` | `domain`, `oracle-client`, Gateway DTO, frontend Zod | `to_oracle_card()` in checkout |
| `FundStatus` | `domain`, `oracle/funds` | Manual From/Into |

> Duplication reflects wire vs domain boundaries. Refactor only if Phase 6 demonstrates high operational cost.

---

## 9. Resolution matrix vs Phase 6

| Phase 6 step | Debt that may activate |
|--------------|------------------------|
| **6.1** Playwright E2E | DT-P1-01, DT-P0-03, DT-P0-04 |
| **6.2** Cross-service | DT-P0-01, DT-P1-03, DT-P1-05 |
| **6.5** Performance | PERF-R-01 async Solana UX | Post-MVP |
| **6.6** CI | DT-P0-02 |
| **6.7** Runbook | DT-P0-03, DT-P0-04 |
| **6.8** Critical debt | DT-P1-01, DT-P1-03 (partial) — [Cierre-Deuda-Fase-6.8-en.md](./Cierre-Deuda-Fase-6.8-en.md) |

---

## 10. Change log

| Date | Change |
|------|--------|
| 2026-07-26 | Playwright E2E 6/6 verified locally — [Pruebas-en.md](./Pruebas-en.md); Gateway CORS; runbook §8 Solana |
| 2026-07-26 | Phase 6.8 — [Cierre-Deuda-Fase-6.8-en.md](./Cierre-Deuda-Fase-6.8-en.md); canonical fixtures; consolidated mocks |
| 2026-07-26 | Runbook 6.7 — [Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md) + `scripts/check-env.sh` |
| 2026-07-26 | Unified CI 6.6 — `.github/workflows/ci.yml` + [Doc/CI-en.md](./CI-en.md) |
| 2026-07-26 | Security review 6.4 — [Revision-Seguridad-Fase-6-en.md](./Revision-Seguridad-Fase-6-en.md); IDOR + idempotency PCI fixes |
| 2026-07-26 | DT-P0-01 mitigated — `cross_service_integration.rs` + `oracle/src/test_support.rs` |

---

## References

- [Doc/Pruebas-en.md](./Pruebas-en.md)
- [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening)
- [Arquitectura-en.md §4](./Arquitectura-en.md#4-reglas-de-dependencia)
- [Acta-Cierre-Fase-5-en.md](./Acta-Cierre-Fase-5-en.md)
- [Acta-Cierre-Fase-4-en.md](./Acta-Cierre-Fase-4-en.md)
