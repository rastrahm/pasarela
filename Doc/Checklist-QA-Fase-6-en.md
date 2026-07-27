# QA Checklist — Phase 6 (UC-01 to UC-11)

> Systematic use case verification before Phase 6 gate.  
> Reference: [Casos-de-Uso-ER-Flujos-en.md](./Casos-de-Uso-ER-Flujos-en.md) · Plan §10 step 6.3  
> **Date:** 2026-07-26 · **Status:** Under review

---

## 1. Coverage summary

| UC | Name | Auto | Partial | Manual | Pending |
|----|------|------|---------|--------|---------|
| UC-01 | Card checkout | 12 | 2 | 1 | 1 |
| UC-02 | Select rail | 6 | 1 | 0 | 1 |
| UC-03 | Validate card | 8 | 1 | 0 | 1 |
| UC-04 | Authorize hold | 9 | 1 | 0 | 1 |
| UC-05 | Settle — Bank | 4 | 0 | 0 | 1 |
| UC-06 | Settle — Binance | 3 | 1 | 1 | 2 |
| UC-07 | Settle — Solana | 4 | 1 | 1 | 3 |
| UC-08 | Rail fallback | 5 | 1 | 0 | 1 |
| UC-09 | Query transaction | 4 | 0 | 1 | 1 |
| UC-10 | View transaction log | 8 | 1 | 1 | 0 |
| UC-11 | Oracle access control | 11 | 0 | 0 | 1 |

**Legend:** ✅ Automated · 🔄 Partial (mock/stub) · ⬜ Manual · ⏭ Pending post-MVP

**Quick regression commands:**

```bash
# Backend (mock Oracle + stub settlement)
cargo test -p api-gateway
cargo test -p oracle-authorization -- --test-threads=1   # requires PostgreSQL

# Cross-service (real Oracle)
export ORACLE_DATABASE_URL=postgres://postgres:<pass>@localhost:5432/oracle_test
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# Frontend
cd frontend && pnpm test:run

# Canonical fixture contract
cargo test -p api-gateway --test contract_fixtures

# Playwright E2E — full stack (6 tests)
cd tests/e2e && pnpm test

# Anchor (UC-07 on-chain)
cd programs/payment-settlement && anchor test
```

---

## 2. UC-01 — Perform card checkout

**Actor:** Buyer · **Endpoint:** `POST /api/v1/checkout`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-01-01 | Successful checkout — traditional bank | `200`, `status: settled`, proof `ACH-*` | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable`, `cross_service_integration::cross_service_full_checkout_settled_and_queryable` |
| UC-01-02 | Successful checkout — Binance CEX | `200`, proof `CEX-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-01-03 | Successful checkout — Solana | `200`, proof `SOL-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-01-04 | Zod validation fails in frontend | Field errors; no `fetch` | ✅ | `CardForm.test.tsx`, `CheckoutPage.interaction.test.tsx` |
| UC-01-05 | Invalid merchant API key | `401 UNAUTHORIZED` | ✅ | `e2e_integration::gate_checkout_requires_bearer_token`, `checkout_integration::checkout_rejects_invalid_api_key` |
| UC-01-06 | Missing Idempotency-Key | `422` | ✅ | `e2e_integration::gate_checkout_requires_idempotency_key` |
| UC-01-07 | Duplicate Idempotency-Key (same body) | `200` idempotent; single Oracle authorize | ✅ | `e2e_integration::gate_idempotent_replay_returns_same_transaction`, `cross_service_integration::cross_service_idempotent_replay_single_oracle_authorize` |
| UC-01-08 | Duplicate Idempotency-Key (different body) | `409 CONFLICT` | ✅ | `idempotency_integration::same_idempotency_key_with_different_body_returns_409` |
| UC-01-09 | Invalid Luhn / unknown brand | `422 INVALID_CARD` | ✅ | `cross_service_integration::cross_service_invalid_card_from_real_oracle`, `error_mapping_integration::oracle_invalid_card_returns_422` |
| UC-01-10 | Insufficient funds | `402 INSUFFICIENT_FUNDS` | ✅ | `cross_service_integration::cross_service_insufficient_funds_from_real_oracle`, `error_mapping_integration::oracle_insufficient_funds_returns_402` |
| UC-01-11 | Antifraud decline (UC-12) | `402` | 🔄 | `oracle/tests/antifraud_integration.rs` (Oracle direct; not Gateway E2E) |
| UC-01-12 | Rail unavailable | `503 RAIL_UNAVAILABLE` | ✅ | `error_mapping_integration::no_viable_rail_returns_503` |
| UC-01-13 | Internal settlement error | `500`; hold released | ✅ | `e2e_integration::gate_hold_released_when_settlement_fails`, `cross_service_integration::cross_service_hold_released_when_settlement_fails` |
| UC-01-14 | Invalid amount (≤ 0) | `422` Gateway | ✅ | `checkout_integration::checkout_rejects_invalid_amount` |
| UC-01-15 | Manual browser checkout | UI settled + receipt | ✅ | Playwright `checkout exitoso — riel *` (2026-07-26, local stack) |
| UC-01-16 | Checkout latency ~1–3 s | No UX timeout | ⏭ | Step 6.5 performance |

**Manual UC-01-15 verification:**

1. Start Oracle + Gateway + frontend (`pnpm dev`).
2. Use test data → Validate card → Confirm payment.
3. Confirm receipt in `TransactionViewer`.

---

## 3. UC-02 — Select settlement rail

**Component:** `RailSelector` · **Field:** `funding_type`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-02-01 | Renders 3 rails | Bank, Binance, Solana visible | ✅ | `RailSelector.test.tsx` |
| UC-02-02 | Selection changes `funding_type` in payload | Correct snake_case value | ✅ | `CheckoutPage.interaction.test.tsx` |
| UC-02-03 | Distinct proof per rail in UI | ACH / CEX / SOL labels | ✅ | `CheckoutPage.interaction.test.tsx`, `TransactionViewer.test.tsx` |
| UC-02-04 | Rail sent to Oracle | `funding_type` in authorize | ✅ | `rail_oracle_integration::oracle_receives_funding_type_from_rail_selection` |
| UC-02-05 | No preference → merchant default | `solana_wallet` if configured | ✅ | `e2e_integration::gate_merchant_default_rail_when_checkout_has_no_preference` |
| UC-02-06 | No preference → system default | `traditional_bank` | ✅ | `funding.test.ts` (`DEFAULT_FUNDING_TYPE`) |
| UC-02-07 | Disabled rail in config | Submit blocked / warning | ⏭ | Not implemented in MVP UI |
| UC-02-08 | Accessible selector (radiogroup) | Keyboard navigation | 🔄 | `RailSelector.test.tsx` (radiogroup); partial Playwright E2E |

---

## 4. UC-03 — Validate card (Luhn + brand)

**Service:** Oracle · **Module:** `oracle/src/validation/`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-03-01 | Valid Visa PAN (4111…) | `brand: visa`, `brand_code: 1` | ✅ | `gateway_contract::gateway_client_authorize_and_release_full_flow` |
| UC-03-02 | Invalid Luhn | `INVALID_CARD`; no hold | ✅ | `gateway_contract::gateway_client_invalid_card_maps_contract_error`, `security_fail_closed::invalid_card_does_not_create_hold` |
| UC-03-03 | Unrecognized brand | `INVALID_CARD` | ✅ | `card.test.ts` (frontend UX), Oracle implicit |
| UC-03-04 | Tokenized PAN; not persisted | Only `token_hash` in DB | ✅ | `security_fail_closed::approved_authorization_persists_token_hash_not_pan` |
| UC-03-05 | No PII in logs | PAN/CVV absent | ✅ | `logging_no_pii.rs` |
| UC-03-06 | Frontend validation (UX) | Luhn before submit | ✅ | `card.test.ts`, `CardForm.test.tsx` |
| UC-03-07 | UC-11 rejects before Luhn | `401`/`403`/`429`; no hold | ✅ | `security_fail_closed.rs`, `auth_security.rs` |
| UC-03-08 | Cross-service Gateway→Oracle validation | `422` on checkout | ✅ | `cross_service_integration::cross_service_invalid_card_from_real_oracle` |
| UC-03-09 | Mastercard / Amex prefixes | Brand detected | 🔄 | Oracle unit tests (`validation/mod.rs` mod tests) |
| UC-03-10 | Invalid / expired expiry | Rejection | ⏭ | Post-MVP |

---

## 5. UC-04 — Authorize funds hold

**Service:** Oracle · **Module:** `oracle/src/funds/`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-04-01 | Hold created with sufficient funds | `hold_id` returned; DB row | ✅ | `hold_persistence::authorize_persists_hold_and_release_is_idempotent` |
| UC-04-02 | Insufficient funds | `INSUFFICIENT_FUNDS`; no active hold | ✅ | `hold_persistence::authorize_rejects_insufficient_funds`, `security_fail_closed::insufficient_funds_does_not_create_active_hold` |
| UC-04-03 | Idempotent release | Second release OK | ✅ | `gateway_contract::gateway_client_release_is_idempotent` |
| UC-04-04 | Release unknown hold | `NOT_FOUND` | ✅ | `hold_persistence::release_unknown_hold_returns_not_found` |
| UC-04-05 | Hold released after settlement failure | `status: released` in DB | ✅ | `hold_release_integration::settlement_failure_triggers_oracle_hold_release`, `cross_service_integration::cross_service_hold_released_when_settlement_fails` |
| UC-04-06 | Successful settlement does not release hold | `release_calls = 0` | ✅ | `hold_release_integration::successful_settlement_does_not_release_hold` |
| UC-04-07 | Binance balance with spread buffer (D4) | Hold if balance × (1 − buffer) ≥ amount | 🔄 | `rail_adapters_integration::authorize_with_binance_rail_uses_http_balance` |
| UC-04-08 | Solana RPC not responding | `RAIL_UNAVAILABLE`; fail closed | ✅ | `rail_adapters_integration::authorize_fail_closed_when_rail_unavailable`, `security_fail_closed::rail_unavailable_does_not_create_active_hold` |
| UC-04-09 | Audit log without PII | Event recorded | ✅ | `logging_no_pii.rs` |
| UC-04-10 | Post-settlement hold consume | `POST /hold/consume` | ⏭ | Planned v1.1 — hold remains active after MVP settle |

---

## 6. UC-05 — Settle — Traditional Rail

**Module:** `settlement-adapters` (MVP stub)

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-05-01 | Stub settlement generates bank ref | Proof `ACH-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-05-02 | Transaction → `Settled` | Status in response and GET | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable` |
| UC-05-03 | Cross-service with real Oracle | Same behavior | ✅ | `cross_service_integration::cross_service_three_rails_settle_with_distinct_proofs` |
| UC-05-04 | Real ISO 20022 file | File generated | ⏭ | Post-MVP — stub in MVP |

---

## 7. UC-06 — Settle — Binance CEX Rail

**Services:** `settlement-adapters`, `binance-sim/`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-06-01 | CEX stub settlement | Proof `CEX-MEM-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-06-02 | Simulator HTTP debit | `POST /spot/debit` | 🔄 | `settlement-adapters` unit tests; E2E with binance-sim ⬜ |
| UC-06-03 | Binance API timeout | Hold released; `503` | ⏭ | Pending binance-sim down integration test |
| UC-06-04 | Balance changed between hold and settle | Rejection; hold released | ⏭ | Post-MVP |
| UC-06-05 | E2E with binance-sim running | Real Binance checkout | ✅ | Playwright `checkout exitoso — riel binance_cex` (2026-07-26) |

---

## 8. UC-07 — Settle — Solana Rail (On-Chain)

**Program:** `programs/payment-settlement/` · **Adapter:** `settlement-adapters/solana`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-07-01 | Solana stub settlement | Proof `SOL-MEM-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-07-02 | `process_payment` OK | Tx + `PaymentProcessed` event | ✅ | `programs/payment-settlement/tests/process-payment.ts` |
| UC-07-03 | Invalid signer rejected | On-chain error | ✅ | `process-payment-constraints.ts` |
| UC-07-04 | Integer overflow | Program rejects | ✅ | `process-payment-constraints.ts` |
| UC-07-05 | `finalized` commitment (D10) | Waits for confirmation | 🔄 | Impl in adapter; manual devnet test |
| UC-07-06 | Full devnet E2E | Checkout → real tx | ⬜ | Requires devnet RPC + funded wallet |
| UC-07-07 | Tx does not reach finalized (timeout) | `Pending`; hold not consumed | ⏭ | Post-MVP |
| UC-07-08 | Event without PII on-chain | Only amount/rail/brand_code | ✅ | `settlement-state.ts`, `state.rs` |

---

## 9. UC-08 — Rail fallback (D3)

**Module:** `rail-switcher`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-08-01 | Preferred rail without funds → next | `binance_cex` settled | ✅ | `e2e_integration::gate_rail_fallback_selects_next_viable_rail`, `rail_oracle_integration::fallback_selects_next_rail_and_authorizes_with_it` |
| UC-08-02 | Oracle receives new `funding_type` | Fallback rail type | ✅ | `rail_oracle_integration::fallback_selects_next_rail_and_authorizes_with_it` |
| UC-08-03 | Fallback disabled + no funds | `402` | ✅ | `error_mapping_integration::fallback_disabled_and_preferred_lacks_funds_returns_402` |
| UC-08-04 | No viable rail | `503` or `Failed` | ✅ | `error_mapping_integration::no_viable_rail_returns_503` |
| UC-08-05 | Priority per `RAIL_CONFIG` | Order respected | 🔄 | `rail-switcher` unit tests |
| UC-08-06 | Fallback with real Oracle | Same cross-service flow | ⏭ | Extend `cross_service_integration` with custom RailContext |

---

## 10. UC-09 — Query transaction status

**Endpoint:** `GET /api/v1/transactions/{id}`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-09-01 | Existing transaction | `200`, status, rail, proof | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable`, `checkout_integration::get_transaction_returns_checkout_result` |
| UC-09-02 | Unknown ID | `404 NOT_FOUND` | ✅ | `http_integration::get_transaction_returns_not_found_for_unknown_id` |
| UC-09-03 | No API key | `401` | ✅ | `http_integration::get_transaction_requires_api_key` |
| UC-09-04 | Frontend client ready | `fetchTransaction` in API | ✅ | `gateway.test.ts` |
| UC-09-05 | UI query screen | Dedicated view | ⏭ | Post-MVP (Phase 5 Act §7) |
| UC-09-06 | Manual curl query | Correct JSON response | ⬜ | `crates/api-gateway/README.md` |

---

## 11. UC-10 — View transaction log

**Components:** `TransactionViewer`, `useTransactionLog`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-10-01 | Successful flow messages | Validate → Authorize → Hold → Settle → Complete | ✅ | `transaction.test.ts`, `TransactionViewer.test.tsx` |
| UC-10-02 | Bank receipt | Label «Bank reference» | ✅ | `TransactionViewer.test.tsx` |
| UC-10-03 | Binance receipt | Label «Order ID» | ✅ | `TransactionViewer.test.tsx` (implicit) |
| UC-10-04 | Solana receipt | Label «Tx Signature» | ✅ | `TransactionViewer.test.tsx` |
| UC-10-05 | Error entry in log | `error` level + UX message | ✅ | `CheckoutPage.interaction.test.tsx` |
| UC-10-06 | `aria-live` region | Accessible update | ✅ | `TransactionViewer.test.tsx` |
| UC-10-07 | 402/422/503 UX errors | `CheckoutErrorAlert` | ✅ | `CheckoutErrorAlert.test.tsx`, `errors.test.ts` |
| UC-10-08 | Log in manual checkout | Visible in browser | ⬜ | Manual verification + Playwright (6.1) |
| UC-10-09 | es-AR locale timestamps | Local time format | 🔄 | `TransactionViewer.tsx` — no explicit test |

---

## 12. UC-11 — Oracle access control

**Module:** `oracle/src/auth/`

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-11-01 | Missing API Key | `401`; no hold | ✅ | `auth_security::internal_route_rejects_missing_api_key`, `security_fail_closed::unauthorized_authorize_does_not_create_hold` |
| UC-11-02 | Incorrect API Key | `401` | ✅ | `auth_security::internal_route_rejects_invalid_api_key`, `gateway_contract::gateway_client_unauthorized_with_invalid_api_key` |
| UC-11-03 | IP outside allowlist | `403 FORBIDDEN` | ✅ | `auth_security::internal_route_rejects_ip_not_in_allowlist`, `gateway_contract::gateway_client_forbidden_when_ip_not_in_allowlist` |
| UC-11-04 | Rate limit exceeded | `429`; no hold | ✅ | `rate_limit_integration.rs`, `security_fail_closed::rate_limit_blocks_authorize_without_creating_hold` |
| UC-11-05 | Rate limit per API key + IP | Independent windows | ✅ | `rate_limit_integration::rate_limit_is_per_api_key_and_ip` |
| UC-11-06 | Docker CIDR allowlist | IP 172.17.x accepted | ✅ | `security_fail_closed::cidr_allowlist_accepts_ip_in_docker_prefix` |
| UC-11-07 | Public `/health` without auth | `200 ok` | ✅ | `health_integration::health_returns_ok_without_auth` |
| UC-11-08 | Release requires API key | `401` without key | ✅ | `security_fail_closed::release_route_requires_api_key` |
| UC-11-09 | Fail closed — antifraud decline | No active hold | ✅ | `security_fail_closed::antifraud_decline_does_not_create_active_hold` |
| UC-11-10 | Fail closed — rail down | No active hold | ✅ | `security_fail_closed::rail_unavailable_does_not_create_active_hold` |
| UC-11-11 | Gateway as sole caller | Gateway only in prod | ⏭ | Operational verification Phase 7 (mTLS D7) |

---

## 13. Appendix — UC-12 Antifraud (UC-01 reference)

| ID | Scenario | Expected result | Status | Evidence |
|----|----------|-----------------|--------|----------|
| UC-12-01 | Approved score | Hold created | ✅ | `antifraud_integration` (mock approve) |
| UC-12-02 | Decline score | `402`; no hold | ✅ | `antifraud_integration::authorize_rejects_when_antifraud_declines` |
| UC-12-03 | Antifraud timeout | Fail closed | ✅ | `antifraud_integration::authorize_fail_closed_when_antifraud_unavailable` |
| UC-12-04 | Invalid antifraud API key | Fail closed | ✅ | `antifraud/tests/score_integration::score_requires_api_key` |

---

## 14. UC → component traceability matrix

| UC | Gateway | Oracle | Frontend | Settlement | On-chain |
|----|---------|--------|----------|------------|----------|
| UC-01 | ✅ | ✅ | ✅ | stub | — |
| UC-02 | ✅ | — | ✅ | — | — |
| UC-03 | mapping | ✅ | UX | — | — |
| UC-04 | release | ✅ | — | — | — |
| UC-05 | ✅ | hold | — | stub | — |
| UC-06 | ✅ | hold | — | stub/HTTP | — |
| UC-07 | ✅ | hold | — | stub/RPC | ✅ tests |
| UC-08 | ✅ | ✅ | — | — | — |
| UC-09 | ✅ | — | API | — | — |
| UC-10 | — | — | ✅ | — | — |
| UC-11 | caller | ✅ | — | — | — |

---

## 15. Prioritized pending items (post-checklist)

| Priority | Item | UC | Phase 6 step |
|----------|------|-----|--------------|
| P0 | Playwright E2E 3 rails with real stack | UC-01, UC-10 | 6.1 ✅ local 2026-07-26 — see [Pruebas-en.md](./Pruebas-en.md) |
| P0 | Local stack runbook | All | 6.7 ✅ [Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md) |
| P1 | E2E binance-sim + real HTTP settlement | UC-06 | 6.2 extension |
| P1 | UC-09 screen | UC-09 | Post-MVP |
| P2 | `POST /hold/consume` | UC-04 | Oracle v1.1 |
| P2 | Disabled rail in UI | UC-02 | Post-MVP |
| P2 | Latency performance test | UC-01 | 6.5 |

---

## 16. Approval (Phase 6 gate)

| Role | Criterion | Status | Date | Signature |
|------|-----------|--------|-------|-----------|
| QA / Dev | UC-01–UC-11 checklist reviewed | ⬜ Pending | | |
| QA / Dev | Automated regression green | ⬜ Pending | | |
| QA / Dev | ⬜ manual items verified | ⬜ Pending | | |
| Product | Phase 6 → Phase 7 gate approval | ⬜ Pending | | |

**6.3 closure criterion:** all ✅ or 🔄 items documented; ⬜ manual items executed at least once with local stack; ⏭ items explicitly deferred.

---

## References

- [Pruebas-en.md](./Pruebas-en.md)
- [Casos-de-Uso-ER-Flujos-en.md](./Casos-de-Uso-ER-Flujos-en.md)
- [Plan-de-Implementacion-en.md §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening)
- [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)
- [Acta-Cierre-Fase-5-en.md](./Acta-Cierre-Fase-5-en.md)
- `qa.cursorrules`
