# Phase Closure Record — Phase 2 (Authorization Oracle)

> Gate **2.11** · Technical verification and stakeholder confirmation  
> Date: **2026-07-25**  
> Project: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaration

**Phase 2 — Authorization Oracle (independent service)** is declared **closed**, with the `oracle/` microservice operational, API v1 contract, Gateway client (`oracle-client`), antifraud integration, and verified security test suite.

Advancement toward **Phase 3 (Solana/Anchor)** and/or **Phase 4 (API Gateway)** is authorized, per team priority (may be partially parallelized).

---

## 2. Verified deliverables

| Deliverable | Location | Status |
|-------------|----------|--------|
| API v1 contract | `oracle/docs/API-v1.md`, `oracle/openapi/v1.yaml` | ✅ |
| Hold persistence + audit log | `oracle/migrations/001_init.sql`, `oracle/src/persistence/` | ✅ |
| Hold TTL | `oracle/src/ttl/` | ✅ |
| Hold release | `POST /internal/v1/hold/release` | ✅ |
| Rate limiting | `oracle/src/auth/rate_limit.rs` | ✅ |
| Rail adapters | `oracle/src/rail_adapters/`, `binance-sim/` | ✅ |
| Antifraud client | `oracle/src/antifraud_client/`, `antifraud/` | ✅ |
| PII-free logging | `oracle/src/logging/` | ✅ |
| Crate `oracle-client` | `crates/oracle-client/` | ✅ |
| Gateway ↔ Oracle tests | `oracle/tests/gateway_contract.rs` | ✅ |
| Environment variables | `oracle/.env.example` | ✅ |

---

## 3. Step checklist (Plan §6)

| # | Step | Result |
|---|------|--------|
| 2.1 | API v1 contract | ✅ |
| 2.2 | Hold persistence | ✅ |
| 2.3 | TTL expiration | ✅ |
| 2.4 | `POST /hold/release` | ✅ |
| 2.5 | Rate limiting | ✅ |
| 2.6 | Per-rail queries | ✅ |
| 2.7 | Simulated antifraud | ✅ |
| 2.8 | Structured logging without PII | ✅ |
| 2.9 | Extended security tests | ✅ |
| 2.10 | `oracle-client` crate | ✅ |
| 2.11 | Gateway ↔ Oracle contract tests | ✅ |

---

## 4. Acceptance criteria (gate)

| Criterion | Verification | Result |
|-----------|--------------|--------|
| Green `cargo test` in `oracle/` | 67 tests (`--test-threads=1`) | ✅ |
| Green `cargo test` in `oracle-client` | 13 tests | ✅ |
| PAN never persisted | `approved_authorization_persists_token_hash_not_pan`, migrations | ✅ |
| Logs/audit without PII | `logging_no_pii` (2 tests) | ✅ |
| No API key → 401 | `unauthorized_authorize_does_not_create_hold`, `gateway_contract` | ✅ |
| Invalid IP → 403 | `forbidden_ip_does_not_create_hold`, `gateway_contract` | ✅ |
| Hold created and released | `hold_persistence`, `gateway_client_authorize_and_release_full_flow` | ✅ |
| Fail closed (funds, antifraud, rail) | `security_fail_closed` (10 tests) | ✅ |
| Stakeholder confirmation | Formal gate | ✅ 2026-07-25 |

### Note on `hold/consume`

The `POST /internal/v1/hold/consume` endpoint is planned for **Phase 4** (Gateway, API v1.1). Persistence already supports `consumed` status via `update_status_in_tx`; it does not block Phase 2 closure.

---

## 5. Technical verification executed

```bash
# Oracle (requires PostgreSQL)
cd oracle
ORACLE_DATABASE_URL='postgres://postgres:postgre@localhost:5432/oracle' \
  cargo test -- --test-threads=1

# Gateway client
cargo test -p oracle-client

# Antifraud (auxiliary service D11)
cd antifraud && cargo test
```

| Component | Tests | Result |
|-----------|-------|--------|
| `oracle/` (unit) | 30 | ✅ |
| `oracle/` (integration) | 37 | ✅ |
| `oracle-client` | 13 | ✅ |
| `antifraud` | 7 | ✅ |
| **Phase 2 total** | **87** | ✅ |

### Oracle integration suites

| File | Tests | Area |
|------|-------|------|
| `security_fail_closed.rs` | 10 | Allowlist, rate limit, fail closed |
| `gateway_contract.rs` | 7 | Gateway ↔ Oracle contract |
| `hold_persistence.rs` | 3 | Creation, release, funds |
| `logging_no_pii.rs` | 2 | No PAN/CVV in logs |
| `rail_adapters_integration.rs` | 2 | HTTP/RPC rails |
| `antifraud_integration.rs` | 3 | Antifraud fail closed |
| `rate_limit_integration.rs` | 2 | 429 by API key + IP |
| `auth_security.rs` | 4 | Basic UC-11 |
| `health_integration.rs` | 1 | Healthcheck |
| `api_contract.rs` | 4 | Contract fixtures |

---

## 6. Oracle endpoints (Phase 2 MVP)

| Method | Route | Auth |
|--------|-------|------|
| `GET` | `/health` | No |
| `POST` | `/internal/v1/authorize` | X-API-KEY + allowlist + rate limit |
| `POST` | `/internal/v1/hold/release` | Same |

---

## 7. Documented deviations

| Item | Original plan | Current state | Impact |
|------|---------------|---------------|--------|
| Workspace includes `oracle/` | Oracle outside pasarela workspace | Unified monorepo with `oracle-client` | Low — logical boundary preserved |
| `hold/consume` HTTP | Gate mentions consume | Endpoint in Phase 4; persistence ready | None for Oracle MVP |
| Docker deployment | Dockerfile in deliverables | Removed — native binaries (`cargo run`) | None — agreed with stakeholder |

---

## 8. Authorization

| Role | Action | Date |
|------|--------|------|
| Technical verification (dev) | Gate 2.11 — 87 green tests | 2026-07-25 |
| Stakeholder / Product | **Confirmed — advance to Phase 3 / Phase 4** | 2026-07-25 |

**Authorized next step:** Phase 3 (Anchor program `payment-settlement`) and/or Phase 4 (`crates/api-gateway/` + settlement adapters).

---

## References

- [Plan-de-Implementacion-en.md §6](./Plan-de-Implementacion-en.md#6-fase-2--oracle-de-autorización-servicio-independiente)
- [API-v1.md](../oracle/docs/API-v1.md)
- [Acta-Cierre-Fase-1-en.md](./Acta-Cierre-Fase-1-en.md)
- [Acta-Cierre-Fase-4-en.md](./Acta-Cierre-Fase-4-en.md) — Phase 4 gate (2026-07-26)
- [Acta-Cierre-Fase-0-en.md](./Acta-Cierre-Fase-0-en.md)
