# Security Review — Phase 6.4

> OWASP Top 10 · Simulated PCI · Oracle boundary (Architecture §9)  
> **Date:** 2026-07-26 · **Status:** Completed with P0 remediations

References: [Arquitectura-en.md §9](./Arquitectura-en.md#9-seguridad) · [Checklist-QA-Fase-6-en.md](./Checklist-QA-Fase-6-en.md) · [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)

---

## 1. Executive summary

| Severity | Before | After 6.4 remediation |
|----------|--------|----------------------|
| **Critical** | 3 | **1** (API key in frontend bundle — accepted MVP demo) |
| **High** | 7 | 7 (planned Phase 7) |
| **Medium** | 6 | 6 |
| **Low** | 4 | 4 |
| **Accepted MVP** | 8 | 10 (+2 P0 fixes) |

**Conclusion:** The Oracle boundary (§9.4) meets fail closed with a solid UC-11 suite. **IDOR on transaction lookup** and **PAN persistence in idempotency** were fixed. The demo frontend with `VITE_GATEWAY_API_KEY` is documented as **development only** until BFF in Phase 7.

---

## 2. Applied remediations (2026-07-26)

### SEC-6.4-01 — IDOR `GET /api/v1/transactions/:id` ✅ Fixed

| Field | Detail |
|-------|--------|
| **Risk** | Merchant A read Merchant B transactions with guessed UUID |
| **Fix** | `merchant_id` filter in persistence and `lookup_transaction` |
| **Files** | `services/transactions.rs`, `persistence/*`, `routes/mod.rs` |
| **Test** | `security_integration::security_transaction_lookup_denies_cross_merchant_idor` |

### SEC-6.4-02 — PAN/CVV in idempotency fingerprint ✅ Fixed

| Field | Detail |
|-------|--------|
| **Risk** | `request_fingerprint` serialized full `CheckoutRequest` (PCI) |
| **Fix** | Fingerprint with `amount`, `currency`, `funding_type`, SHA-256 `pan_hash`, `last_four`, expiry — no CVV/cardholder |
| **Files** | `services/idempotency.rs` |
| **Tests** | `idempotency::request_fingerprint_excludes_pan_cvv_and_cardholder`, `security_integration::security_idempotency_fingerprint_not_stored_in_conflict_message` |

---

## 3. Open findings

### Critical — Accepted MVP demo

| ID | Finding | Current mitigation | Plan |
|----|---------|-------------------|------|
| SEC-C-03 | `VITE_GATEWAY_API_KEY` embedded in JS bundle | `isPlaceholderApiKey()` + dev warning in UI; documented `.env.example` | Phase 7: BFF / server-side checkout; never secrets in `VITE_*` |

### High — Phase 7 / hardening

| ID | Finding | File | Recommendation |
|----|---------|------|----------------|
| SEC-A-01 | Trust in `X-Forwarded-For` without validated proxy | `oracle/src/auth/mod.rs`, `checkout.rs` | IP from `ConnectInfo` or LB-injected header |
| SEC-A-02 | Non-constant-time API key comparison | `oracle/src/auth/mod.rs`, `antifraud/`, `binance-sim/` | `subtle::ConstantTimeEq` |
| SEC-A-03 | Deterministic PAN token (demo hash) | `oracle/src/validation/mod.rs` | HMAC-SHA256 + key in Vault |
| SEC-A-04 | No TLS/mTLS Gateway↔Oracle | Architecture D7 | mTLS Phase 7/8 |
| SEC-A-05 | Default bind `0.0.0.0` | `config.rs` (Gateway, Oracle) | Bind `127.0.0.1` in dev; document prod |
| SEC-A-06 | No IP allowlist on antifraud/binance-sim | `antifraud/src/auth/`, `binance-sim/src/auth/` | Internal network / allowlist |
| SEC-A-07 | PAN in browser memory | Checkout SPA flow | CSP + BFF Phase 7 |

### Medium

| ID | Finding | Recommendation |
|----|---------|----------------|
| SEC-M-01 | No CORS / security headers (CSP, HSTS, X-Frame-Options) | `tower_http` on Gateway |
| SEC-M-02 | `500` errors expose internal detail | Generic message to client |
| SEC-M-03 | Oracle `429` → Gateway `500` | Map to `429`/`503` |
| SEC-M-04 | Gateway without anti-PII log tests | Redact module like Oracle |
| SEC-M-05 | Fallback IP `127.0.0.1` in Oracle without header | Validate in prod |
| SEC-M-06 | SSRF via configurable rail URLs | Validate scheme/host in Phase 7 |

### Low

| ID | Finding | Notes |
|----|---------|-------|
| SEC-B-01 | Public `/health` | Accepted — orchestration |
| SEC-B-02 | Antifraud decline → opaque `402` | Correct to avoid leaking fraud |
| SEC-B-03 | Demo keys in `.env.example` | OK if `.env` not committed |
| SEC-B-04 | No `cargo audit` in CI | Step 6.6 |

---

## 4. Oracle boundary (§9.4) — evaluation

| Control | Status | Tests |
|---------|--------|-------|
| Isolation (frontend never calls Oracle) | ✅ | Architecture + `frontend/src/api/gateway.ts` |
| Required `X-API-KEY` | ✅ | `auth_security.rs` |
| IP/CIDR allowlist | ✅ | `security_fail_closed.rs` |
| Rate limit key+IP | ✅ | `rate_limit_integration.rs` |
| Fail closed (no hold on rejection) | ✅ | UC-11 suite |
| PAN not persisted in Oracle DB | ✅ | `approved_authorization_persists_token_hash_not_pan` |
| Logs without PII | ✅ | `logging_no_pii.rs` |
| Antifraud fail closed | ✅ | `antifraud_integration.rs` |
| mTLS | ⏭ Phase 7 | D7 |

---

## 5. Simulated PCI (§9.6)

| PCI requirement (MVP) | Status |
|-----------------------|--------|
| CVV never persisted | ✅ Not in Gateway/Oracle DB schemas |
| PAN not persisted Gateway | ✅ Idempotency fixed (SEC-6.4-02) |
| PAN not persisted Oracle | ✅ Only `card_token_hash` |
| Isolated CDE (Oracle) | ✅ Internal HTTP boundary |
| In-memory tokenization | 🔄 Demo hash — improve Phase 7 |
| Logs without PAN/CVV | ✅ Oracle; Gateway partial |
| 3DS / SCA | ⏭ Post-MVP D8 |

---

## 6. OWASP Top 10 — MVP status

| ID | Category | Status | Notes |
|----|----------|--------|-------|
| A01 | Broken Access Control | **Improved** | IDOR fixed; Oracle allowlist with XFF caveat |
| A02 | Cryptographic Failures | Partial | Weak PAN token; demo frontend API key |
| A03 | Injection | ✅ | Parameterized sqlx; frontend Zod |
| A04 | Insecure Design | Accepted MVP | Fail closed documented |
| A05 | Security Misconfiguration | Weak | No TLS/CORS/headers; bind 0.0.0.0 |
| A06 | Vulnerable Components | Not evaluated | CI 6.6 pending |
| A07 | Auth Failures | Partial | Auth present; timing side-channel |
| A08 | Software Integrity | Partial | Anchor constraints OK |
| A09 | Logging Failures | Partial | Oracle strong; Gateway improvable |
| A10 | SSRF | Low | Rail URLs via env |

---

## 7. Web3 / on-chain

| Control | Status | Evidence |
|---------|--------|----------|
| `PaymentProcessed` without PII | ✅ | `programs/payment-settlement/src/state.rs` |
| Overflow checked | ✅ | `process-payment-constraints.ts` |
| Signer validation | ✅ | Anchor tests |
| Zero seed phrase in UI | ✅ | Not requested in frontend |

---

## 8. Security test coverage

| Suite | Scope |
|-------|-------|
| `oracle/tests/security_fail_closed.rs` | UC-11 fail closed |
| `oracle/tests/auth_security.rs` | API key, IP |
| `oracle/tests/rate_limit_integration.rs` | 429 |
| `oracle/tests/logging_no_pii.rs` | PCI logs |
| `crates/api-gateway/tests/security_integration.rs` | IDOR, PCI fingerprint |
| `crates/api-gateway/tests/idempotency_integration.rs` | D9 |
| `crates/api-gateway/tests/error_mapping_integration.rs` | HTTP §6.2 |

---

## 9. Post-review action plan

| Priority | Action | Phase |
|----------|--------|-------|
| P0 | ~~Transaction IDOR~~ | ✅ 6.4 |
| P0 | ~~Fingerprint without PAN~~ | ✅ 6.4 |
| P0 | Document frontend API key demo only | ✅ this doc |
| P1 | HMAC PAN token | 7 |
| P1 | Constant-time API keys | 7 |
| P1 | Trusted IP (not client XFF) | 7 |
| P2 | CORS + CSP + HSTS | 7 |
| P2 | Generic 500 errors | 7 |
| P3 | mTLS Gateway↔Oracle | 7/8 |

---

## 10. Approval

| Role | Criterion | Status | Date |
|------|-----------|--------|------|
| Dev / Security | §9 review completed | ✅ | 2026-07-26 |
| Dev / Security | P0 remediations applied | ✅ | 2026-07-26 |
| Dev / Security | `security_integration` tests green | ✅ | 2026-07-26 |
| Product | Acceptance of documented MVP risks | ⬜ | |

---

## References

- [Arquitectura-en.md §9](./Arquitectura-en.md#9-seguridad)
- [Checklist-QA-Fase-6-en.md §12 UC-11](./Checklist-QA-Fase-6-en.md#12-uc-11--oracle-access-control)
- [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening)
- [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)
