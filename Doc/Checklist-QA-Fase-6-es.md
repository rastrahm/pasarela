# Checklist QA — Fase 6 (UC-01 a UC-11)

> Verificación sistemática de casos de uso antes del gate Fase 6.  
> Referencia: [Casos-de-Uso-ER-Flujos.md](./Casos-de-Uso-ER-Flujos.md) · Plan §10 paso 6.3  
> **Fecha:** 2026-07-26 · **Estado:** En revisión

---

## 1. Resumen de cobertura

| UC | Nombre | Auto | Parcial | Manual | Pendiente |
|----|--------|------|---------|--------|-----------|
| UC-01 | Checkout con tarjeta | 12 | 2 | 1 | 1 |
| UC-02 | Seleccionar riel | 6 | 1 | 0 | 1 |
| UC-03 | Validar tarjeta | 8 | 1 | 0 | 1 |
| UC-04 | Autorizar hold | 9 | 1 | 0 | 1 |
| UC-05 | Liquidar — Banco | 4 | 0 | 0 | 1 |
| UC-06 | Liquidar — Binance | 3 | 1 | 1 | 2 |
| UC-07 | Liquidar — Solana | 4 | 1 | 1 | 3 |
| UC-08 | Fallback de riel | 5 | 1 | 0 | 1 |
| UC-09 | Consultar transacción | 4 | 0 | 1 | 1 |
| UC-10 | Visualizar log | 8 | 1 | 1 | 0 |
| UC-11 | Control acceso Oracle | 11 | 0 | 0 | 1 |

**Leyenda:** ✅ Automatizado · 🔄 Parcial (mock/stub) · ⬜ Manual · ⏭ Pendiente post-MVP

**Comandos de regresión rápida:**

```bash
# Backend (mock Oracle + stub settlement)
cargo test -p api-gateway
cargo test -p oracle-authorization -- --test-threads=1   # requiere PostgreSQL

# Cross-service (Oracle real)
export ORACLE_DATABASE_URL=postgres://postgres:<pass>@localhost:5432/oracle_test
cargo test -p api-gateway --test cross_service_integration -- --test-threads=1

# Frontend
cd frontend && pnpm test:run

# Contrato fixtures canónicos
cargo test -p api-gateway --test contract_fixtures

# E2E Playwright — stack completo (6 tests)
cd tests/e2e && pnpm test

# Anchor (UC-07 on-chain)
cd programs/payment-settlement && anchor test
```

---

## 2. UC-01 — Realizar checkout con tarjeta

**Actor:** Comprador · **Endpoint:** `POST /api/v1/checkout`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-01-01 | Checkout exitoso — banco tradicional | `200`, `status: settled`, proof `ACH-*` | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable`, `cross_service_integration::cross_service_full_checkout_settled_and_queryable` |
| UC-01-02 | Checkout exitoso — Binance CEX | `200`, proof `CEX-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-01-03 | Checkout exitoso — Solana | `200`, proof `SOL-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-01-04 | Validación Zod falla en frontend | Errores de campo; sin `fetch` | ✅ | `CardForm.test.tsx`, `CheckoutPage.interaction.test.tsx` |
| UC-01-05 | API key comercio inválida | `401 UNAUTHORIZED` | ✅ | `e2e_integration::gate_checkout_requires_bearer_token`, `checkout_integration::checkout_rejects_invalid_api_key` |
| UC-01-06 | Idempotency-Key ausente | `422` | ✅ | `e2e_integration::gate_checkout_requires_idempotency_key` |
| UC-01-07 | Idempotency-Key duplicada (mismo body) | `200` idempotente; un solo authorize Oracle | ✅ | `e2e_integration::gate_idempotent_replay_returns_same_transaction`, `cross_service_integration::cross_service_idempotent_replay_single_oracle_authorize` |
| UC-01-08 | Idempotency-Key duplicada (body distinto) | `409 CONFLICT` | ✅ | `idempotency_integration::same_idempotency_key_with_different_body_returns_409` |
| UC-01-09 | Luhn inválido / marca desconocida | `422 INVALID_CARD` | ✅ | `cross_service_integration::cross_service_invalid_card_from_real_oracle`, `error_mapping_integration::oracle_invalid_card_returns_422` |
| UC-01-10 | Fondos insuficientes | `402 INSUFFICIENT_FUNDS` | ✅ | `cross_service_integration::cross_service_insufficient_funds_from_real_oracle`, `error_mapping_integration::oracle_insufficient_funds_returns_402` |
| UC-01-11 | Antifraude decline (UC-12) | `402` | 🔄 | `oracle/tests/antifraud_integration.rs` (Oracle directo; no E2E Gateway) |
| UC-01-12 | Riel no disponible | `503 RAIL_UNAVAILABLE` | ✅ | `error_mapping_integration::no_viable_rail_returns_503` |
| UC-01-13 | Error interno settlement | `500`; hold liberado | ✅ | `e2e_integration::gate_hold_released_when_settlement_fails`, `cross_service_integration::cross_service_hold_released_when_settlement_fails` |
| UC-01-14 | Monto inválido (≤ 0) | `422` Gateway | ✅ | `checkout_integration::checkout_rejects_invalid_amount` |
| UC-01-15 | Checkout manual navegador | UI settled + comprobante | ✅ | Playwright `checkout exitoso — riel *` (2026-07-26, stack local) |
| UC-01-16 | Latencia checkout ~1–3 s | Sin timeout UX | ⏭ | Paso 6.5 rendimiento |

**Verificación manual UC-01-15:**

1. Levantar Oracle + Gateway + frontend (`pnpm dev`).
2. Usar datos de prueba → Validar tarjeta → Confirmar pago.
3. Confirmar comprobante en `TransactionViewer`.

---

## 3. UC-02 — Seleccionar riel de liquidación

**Componente:** `RailSelector` · **Campo:** `funding_type`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-02-01 | Renderiza 3 rieles | Banco, Binance, Solana visibles | ✅ | `RailSelector.test.tsx` |
| UC-02-02 | Selección cambia `funding_type` en payload | Valor snake_case correcto | ✅ | `CheckoutPage.interaction.test.tsx` |
| UC-02-03 | Proof distinto por riel en UI | ACH / CEX / SOL labels | ✅ | `CheckoutPage.interaction.test.tsx`, `TransactionViewer.test.tsx` |
| UC-02-04 | Riel enviado al Oracle | `funding_type` en authorize | ✅ | `rail_oracle_integration::oracle_receives_funding_type_from_rail_selection` |
| UC-02-05 | Sin preferencia → default comercio | `solana_wallet` si configurado | ✅ | `e2e_integration::gate_merchant_default_rail_when_checkout_has_no_preference` |
| UC-02-06 | Sin preferencia → default sistema | `traditional_bank` | ✅ | `funding.test.ts` (`DEFAULT_FUNDING_TYPE`) |
| UC-02-07 | Riel deshabilitado en config | Submit bloqueado / aviso | ⏭ | No implementado en MVP UI |
| UC-02-08 | Selector accesible (radiogroup) | Navegación teclado | 🔄 | `RailSelector.test.tsx` (radiogroup); E2E Playwright parcial |

---

## 4. UC-03 — Validar tarjeta (Luhn + marca)

**Servicio:** Oracle · **Módulo:** `oracle/src/validation/`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-03-01 | PAN Visa válido (4111…) | `brand: visa`, `brand_code: 1` | ✅ | `gateway_contract::gateway_client_authorize_and_release_full_flow` |
| UC-03-02 | Luhn inválido | `INVALID_CARD`; sin hold | ✅ | `gateway_contract::gateway_client_invalid_card_maps_contract_error`, `security_fail_closed::invalid_card_does_not_create_hold` |
| UC-03-03 | Marca no reconocida | `INVALID_CARD` | ✅ | `card.test.ts` (frontend UX), Oracle implícito |
| UC-03-04 | PAN tokenizado; no persiste | Solo `token_hash` en BD | ✅ | `security_fail_closed::approved_authorization_persists_token_hash_not_pan` |
| UC-03-05 | Sin PII en logs | PAN/CVV ausentes | ✅ | `logging_no_pii.rs` |
| UC-03-06 | Validación frontend (UX) | Luhn antes de submit | ✅ | `card.test.ts`, `CardForm.test.tsx` |
| UC-03-07 | UC-11 rechaza antes de Luhn | `401`/`403`/`429`; sin hold | ✅ | `security_fail_closed.rs`, `auth_security.rs` |
| UC-03-08 | Validación cross-service Gateway→Oracle | `422` en checkout | ✅ | `cross_service_integration::cross_service_invalid_card_from_real_oracle` |
| UC-03-09 | Mastercard / Amex prefijos | Marca detectada | 🔄 | Tests unitarios Oracle (`validation/mod.rs` mod tests) |
| UC-03-10 | Expiry inválido / vencida | Rechazo | ⏭ | Post-MVP |

---

## 5. UC-04 — Autorizar hold de fondos

**Servicio:** Oracle · **Módulo:** `oracle/src/funds/`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-04-01 | Hold creado con fondos suficientes | `hold_id` retornado; fila en BD | ✅ | `hold_persistence::authorize_persists_hold_and_release_is_idempotent` |
| UC-04-02 | Fondos insuficientes | `INSUFFICIENT_FUNDS`; sin hold activo | ✅ | `hold_persistence::authorize_rejects_insufficient_funds`, `security_fail_closed::insufficient_funds_does_not_create_active_hold` |
| UC-04-03 | Release idempotente | Segundo release OK | ✅ | `gateway_contract::gateway_client_release_is_idempotent` |
| UC-04-04 | Release hold desconocido | `NOT_FOUND` | ✅ | `hold_persistence::release_unknown_hold_returns_not_found` |
| UC-04-05 | Hold liberado tras fallo settlement | `status: released` en BD | ✅ | `hold_release_integration::settlement_failure_triggers_oracle_hold_release`, `cross_service_integration::cross_service_hold_released_when_settlement_fails` |
| UC-04-06 | Settlement exitoso no libera hold | `release_calls = 0` | ✅ | `hold_release_integration::successful_settlement_does_not_release_hold` |
| UC-04-07 | Saldo Binance con spread buffer (D4) | Hold si saldo × (1 − buffer) ≥ monto | 🔄 | `rail_adapters_integration::authorize_with_binance_rail_uses_http_balance` |
| UC-04-08 | RPC Solana no responde | `RAIL_UNAVAILABLE`; fail closed | ✅ | `rail_adapters_integration::authorize_fail_closed_when_rail_unavailable`, `security_fail_closed::rail_unavailable_does_not_create_active_hold` |
| UC-04-09 | Audit log sin PII | Evento registrado | ✅ | `logging_no_pii.rs` |
| UC-04-10 | Hold consume post-settlement | `POST /hold/consume` | ⏭ | Planificado v1.1 — hold queda activo tras settle MVP |

---

## 6. UC-05 — Liquidar — Riel Tradicional

**Módulo:** `settlement-adapters` (stub MVP)

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-05-01 | Settlement stub genera ref bancaria | Proof `ACH-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-05-02 | Transacción → `Settled` | Status en respuesta y GET | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable` |
| UC-05-03 | Cross-service con Oracle real | Mismo comportamiento | ✅ | `cross_service_integration::cross_service_three_rails_settle_with_distinct_proofs` |
| UC-05-04 | Archivo ISO 20022 real | Archivo generado | ⏭ | Post-MVP — stub en MVP |

---

## 7. UC-06 — Liquidar — Riel Binance CEX

**Servicios:** `settlement-adapters`, `binance-sim/`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-06-01 | Settlement stub CEX | Proof `CEX-MEM-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-06-02 | Débito HTTP simulador | `POST /spot/debit` | 🔄 | `settlement-adapters` unit tests; E2E con binance-sim ⬜ |
| UC-06-03 | API Binance timeout | Hold liberado; `503` | ⏭ | Pendiente test integración binance-sim caído |
| UC-06-04 | Saldo cambió entre hold y settle | Rechazo; hold liberado | ⏭ | Post-MVP |
| UC-06-05 | E2E con binance-sim levantado | Checkout real Binance | ✅ | Playwright `checkout exitoso — riel binance_cex` (2026-07-26) |

---

## 8. UC-07 — Liquidar — Riel Solana (On-Chain)

**Programa:** `programs/payment-settlement/` · **Adapter:** `settlement-adapters/solana`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-07-01 | Settlement stub Solana | Proof `SOL-MEM-*` | ✅ | `e2e_integration::gate_three_rails_settle_with_distinct_proofs` |
| UC-07-02 | `process_payment` OK | Tx + evento `PaymentProcessed` | ✅ | `programs/payment-settlement/tests/process-payment.ts` |
| UC-07-03 | Signer inválido rechazado | Error on-chain | ✅ | `process-payment-constraints.ts` |
| UC-07-04 | Integer overflow | Programa rechaza | ✅ | `process-payment-constraints.ts` |
| UC-07-05 | Commitment `finalized` (D10) | Espera confirmación | 🔄 | Impl en adapter; test devnet manual |
| UC-07-06 | E2E devnet completo | Checkout → tx real | ⬜ | Requiere RPC devnet + wallet funded |
| UC-07-07 | Tx no alcanza finalized (timeout) | `Pending`; hold no consumido | ⏭ | Post-MVP |
| UC-07-08 | Evento sin PII on-chain | Solo amount/rail/brand_code | ✅ | `settlement-state.ts`, `state.rs` |

---

## 9. UC-08 — Fallback de riel (D3)

**Módulo:** `rail-switcher`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-08-01 | Riel preferido sin fondos → siguiente | `binance_cex` liquidado | ✅ | `e2e_integration::gate_rail_fallback_selects_next_viable_rail`, `rail_oracle_integration::fallback_selects_next_rail_and_authorizes_with_it` |
| UC-08-02 | Oracle recibe nuevo `funding_type` | Tipo del riel fallback | ✅ | `rail_oracle_integration::fallback_selects_next_rail_and_authorizes_with_it` |
| UC-08-03 | Fallback deshabilitado + sin fondos | `402` | ✅ | `error_mapping_integration::fallback_disabled_and_preferred_lacks_funds_returns_402` |
| UC-08-04 | Ningún riel viable | `503` o `Failed` | ✅ | `error_mapping_integration::no_viable_rail_returns_503` |
| UC-08-05 | Prioridad según `RAIL_CONFIG` | Orden respetado | 🔄 | `rail-switcher` unit tests |
| UC-08-06 | Fallback con Oracle real | Mismo flujo cross-service | ⏭ | Extender `cross_service_integration` con RailContext custom |

---

## 10. UC-09 — Consultar estado de transacción

**Endpoint:** `GET /api/v1/transactions/{id}`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-09-01 | Transacción existente | `200`, status, rail, proof | ✅ | `e2e_integration::gate_full_checkout_flow_settled_and_queryable`, `checkout_integration::get_transaction_returns_checkout_result` |
| UC-09-02 | ID desconocido | `404 NOT_FOUND` | ✅ | `http_integration::get_transaction_returns_not_found_for_unknown_id` |
| UC-09-03 | Sin API key | `401` | ✅ | `http_integration::get_transaction_requires_api_key` |
| UC-09-04 | Cliente frontend listo | `fetchTransaction` en API | ✅ | `gateway.test.ts` |
| UC-09-05 | Pantalla UI consulta | Vista dedicada | ⏭ | Post-MVP (Acta Fase 5 §7) |
| UC-09-06 | Consulta manual curl | Respuesta JSON correcta | ⬜ | `crates/api-gateway/README.md` |

---

## 11. UC-10 — Visualizar log de transacción

**Componentes:** `TransactionViewer`, `useTransactionLog`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-10-01 | Mensajes flujo exitoso | Validar → Autorizar → Hold → Liquidar → Completada | ✅ | `transaction.test.ts`, `TransactionViewer.test.tsx` |
| UC-10-02 | Comprobante banco | Label «Referencia bancaria» | ✅ | `TransactionViewer.test.tsx` |
| UC-10-03 | Comprobante Binance | Label «Order ID» | ✅ | `TransactionViewer.test.tsx` (implícito) |
| UC-10-04 | Comprobante Solana | Label «Tx Signature» | ✅ | `TransactionViewer.test.tsx` |
| UC-10-05 | Entrada error en log | Nivel `error` + mensaje UX | ✅ | `CheckoutPage.interaction.test.tsx` |
| UC-10-06 | Región `aria-live` | Actualización accesible | ✅ | `TransactionViewer.test.tsx` |
| UC-10-07 | Errores 402/422/503 UX | `CheckoutErrorAlert` | ✅ | `CheckoutErrorAlert.test.tsx`, `errors.test.ts` |
| UC-10-08 | Log en checkout manual | Visible en navegador | ⬜ | Verificación manual + Playwright (6.1) |
| UC-10-09 | Timestamps locale es-AR | Formato hora local | 🔄 | `TransactionViewer.tsx` — sin test explícito |

---

## 12. UC-11 — Control de acceso al Oracle

**Módulo:** `oracle/src/auth/`

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-11-01 | API Key ausente | `401`; sin hold | ✅ | `auth_security::internal_route_rejects_missing_api_key`, `security_fail_closed::unauthorized_authorize_does_not_create_hold` |
| UC-11-02 | API Key incorrecta | `401` | ✅ | `auth_security::internal_route_rejects_invalid_api_key`, `gateway_contract::gateway_client_unauthorized_with_invalid_api_key` |
| UC-11-03 | IP fuera de allowlist | `403 FORBIDDEN` | ✅ | `auth_security::internal_route_rejects_ip_not_in_allowlist`, `gateway_contract::gateway_client_forbidden_when_ip_not_in_allowlist` |
| UC-11-04 | Rate limit excedido | `429`; sin hold | ✅ | `rate_limit_integration.rs`, `security_fail_closed::rate_limit_blocks_authorize_without_creating_hold` |
| UC-11-05 | Rate limit por API key + IP | Ventanas independientes | ✅ | `rate_limit_integration::rate_limit_is_per_api_key_and_ip` |
| UC-11-06 | Allowlist CIDR Docker | IP 172.17.x aceptada | ✅ | `security_fail_closed::cidr_allowlist_accepts_ip_in_docker_prefix` |
| UC-11-07 | `/health` público sin auth | `200 ok` | ✅ | `health_integration::health_returns_ok_without_auth` |
| UC-11-08 | Release requiere API key | `401` sin key | ✅ | `security_fail_closed::release_route_requires_api_key` |
| UC-11-09 | Fail closed — antifraude decline | Sin hold activo | ✅ | `security_fail_closed::antifraud_decline_does_not_create_active_hold` |
| UC-11-10 | Fail closed — riel caído | Sin hold activo | ✅ | `security_fail_closed::rail_unavailable_does_not_create_active_hold` |
| UC-11-11 | Gateway como único caller | Solo Gateway en prod | ⏭ | Verificación operativa Fase 7 (mTLS D7) |

---

## 13. Apéndice — UC-12 Antifraude (referencia UC-01)

| ID | Escenario | Resultado esperado | Estado | Evidencia |
|----|-----------|-------------------|--------|-----------|
| UC-12-01 | Score aprobado | Hold creado | ✅ | `antifraud_integration` (mock approve) |
| UC-12-02 | Score decline | `402`; sin hold | ✅ | `antifraud_integration::authorize_rejects_when_antifraud_declines` |
| UC-12-03 | Antifraude timeout | Fail closed | ✅ | `antifraud_integration::authorize_fail_closed_when_antifraud_unavailable` |
| UC-12-04 | API key antifraude inválida | Fail closed | ✅ | `antifraud/tests/score_integration::score_requires_api_key` |

---

## 14. Matriz de trazabilidad UC → componente

| UC | Gateway | Oracle | Frontend | Settlement | On-chain |
|----|---------|--------|----------|------------|----------|
| UC-01 | ✅ | ✅ | ✅ | stub | — |
| UC-02 | ✅ | — | ✅ | — | — |
| UC-03 | mapeo | ✅ | UX | — | — |
| UC-04 | release | ✅ | — | — | — |
| UC-05 | ✅ | hold | — | stub | — |
| UC-06 | ✅ | hold | — | stub/HTTP | — |
| UC-07 | ✅ | hold | — | stub/RPC | ✅ tests |
| UC-08 | ✅ | ✅ | — | — | — |
| UC-09 | ✅ | — | API | — | — |
| UC-10 | — | — | ✅ | — | — |
| UC-11 | caller | ✅ | — | — | — |

---

## 15. Items pendientes priorizados (post-checklist)

| Prioridad | Item | UC | Paso Fase 6 |
|-----------|------|-----|-------------|
| P0 | E2E Playwright 3 rieles con stack real | UC-01, UC-10 | 6.1 ✅ local 2026-07-26 — ver [Pruebas.md](./Pruebas.md) |
| P0 | Runbook stack local | Todos | 6.7 ✅ [Runbook-Desarrollo.md](./Runbook-Desarrollo.md) |
| P1 | E2E binance-sim + settlement HTTP real | UC-06 | 6.2 extensión |
| P1 | Pantalla UC-09 | UC-09 | Post-MVP |
| P2 | `POST /hold/consume` | UC-04 | v1.1 Oracle |
| P2 | Riel deshabilitado en UI | UC-02 | Post-MVP |
| P2 | Test rendimiento latencia | UC-01 | 6.5 |

---

## 16. Aprobación (gate Fase 6)

| Rol | Criterio | Estado | Fecha | Firma |
|-----|----------|--------|-------|-------|
| QA / Dev | Checklist UC-01–UC-11 revisado | ⬜ Pendiente | | |
| QA / Dev | Regresión automatizada verde | ⬜ Pendiente | | |
| QA / Dev | Items ⬜ manual verificados | ⬜ Pendiente | | |
| Producto | Aprobación gate Fase 6 → Fase 7 | ⬜ Pendiente | | |

**Criterio de cierre 6.3:** todos los items ✅ o 🔄 documentados; items ⬜ manual ejecutados al menos una vez con stack local; items ⏭ explícitamente diferidos.

---

## Referencias

- [Pruebas.md](./Pruebas.md)
- [Casos-de-Uso-ER-Flujos.md](./Casos-de-Uso-ER-Flujos.md)
- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Deuda-Tecnica.md](./Deuda-Tecnica.md)
- [Acta-Cierre-Fase-5.md](./Acta-Cierre-Fase-5.md)
- `qa.cursorrules`
