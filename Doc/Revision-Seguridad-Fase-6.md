> **Documentation / Documentación:** [Español (es)](Revision-Seguridad-Fase-6-es.md) · [English (en)](Revision-Seguridad-Fase-6-en.md)
>
# Revisión de seguridad — Fase 6.4

> OWASP Top 10 · PCI simulado · Frontera Oracle (Arquitectura §9)  
> **Fecha:** 2026-07-26 · **Estado:** Completada con remediaciones P0

Referencias: [Arquitectura.md §9](./Arquitectura.md#9-seguridad) · [Checklist-QA-Fase-6.md](./Checklist-QA-Fase-6.md) · [Deuda-Tecnica.md](./Deuda-Tecnica.md)

---

## 1. Resumen ejecutivo

| Severidad | Antes | Tras remediación 6.4 |
|-----------|-------|----------------------|
| **Crítico** | 3 | **1** (API key en bundle frontend — aceptado MVP demo) |
| **Alto** | 7 | 7 (planificados Fase 7) |
| **Medio** | 6 | 6 |
| **Bajo** | 4 | 4 |
| **Aceptado MVP** | 8 | 10 (+2 fixes P0) |

**Conclusión:** La frontera Oracle (§9.4) cumple fail closed con suite UC-11 sólida. Se corrigieron **IDOR en consulta de transacciones** y **persistencia de PAN en idempotencia**. El frontend demo con `VITE_GATEWAY_API_KEY` queda documentado como **solo desarrollo** hasta BFF en Fase 7.

---

## 2. Remediaciones aplicadas (2026-07-26)

### SEC-6.4-01 — IDOR `GET /api/v1/transactions/:id` ✅ Corregido

| Campo | Detalle |
|-------|---------|
| **Riesgo** | Comercio A leía transacciones de comercio B con UUID adivinado |
| **Fix** | Filtro `merchant_id` en persistencia y `lookup_transaction` |
| **Archivos** | `services/transactions.rs`, `persistence/*`, `routes/mod.rs` |
| **Test** | `security_integration::security_transaction_lookup_denies_cross_merchant_idor` |

### SEC-6.4-02 — PAN/CVV en huella de idempotencia ✅ Corregido

| Campo | Detalle |
|-------|---------|
| **Riesgo** | `request_fingerprint` serializaba `CheckoutRequest` completo (PCI) |
| **Fix** | Fingerprint con `amount`, `currency`, `funding_type`, `pan_hash` SHA-256, `last_four`, expiry — sin CVV/titular |
| **Archivos** | `services/idempotency.rs` |
| **Tests** | `idempotency::request_fingerprint_excludes_pan_cvv_and_cardholder`, `security_integration::security_idempotency_fingerprint_not_stored_in_conflict_message` |

---

## 3. Hallazgos abiertos

### Crítico — Aceptado MVP demo

| ID | Hallazgo | Mitigación actual | Plan |
|----|----------|-------------------|------|
| SEC-C-03 | `VITE_GATEWAY_API_KEY` embebida en bundle JS | `isPlaceholderApiKey()` + aviso dev en UI; `.env.example` documentado | Fase 7: BFF / checkout server-side; nunca secretos en `VITE_*` |

### Alto — Fase 7 / hardening

| ID | Hallazgo | Archivo | Recomendación |
|----|----------|---------|---------------|
| SEC-A-01 | Confianza en `X-Forwarded-For` sin proxy validado | `oracle/src/auth/mod.rs`, `checkout.rs` | IP desde `ConnectInfo` o header inyectado por LB |
| SEC-A-02 | Comparación API key no constant-time | `oracle/src/auth/mod.rs`, `antifraud/`, `binance-sim/` | `subtle::ConstantTimeEq` |
| SEC-A-03 | Token PAN determinístico (demo hash) | `oracle/src/validation/mod.rs` | HMAC-SHA256 + clave en Vault |
| SEC-A-04 | Sin TLS/mTLS Gateway↔Oracle | Arquitectura D7 | mTLS Fase 7/8 |
| SEC-A-05 | Bind `0.0.0.0` por defecto | `config.rs` (Gateway, Oracle) | Bind `127.0.0.1` en dev; documentar prod |
| SEC-A-06 | Sin allowlist IP en antifraud/binance-sim | `antifraud/src/auth/`, `binance-sim/src/auth/` | Red interna / allowlist |
| SEC-A-07 | PAN en memoria browser | Flujo checkout SPA | CSP + BFF Fase 7 |

### Medio

| ID | Hallazgo | Recomendación |
|----|----------|---------------|
| SEC-M-01 | Sin CORS / headers seguridad (CSP, HSTS, X-Frame-Options) | `tower_http` en Gateway |
| SEC-M-02 | Errores `500` exponen detalle interno | Mensaje genérico al cliente |
| SEC-M-03 | Oracle `429` → Gateway `500` | Mapear a `429`/`503` |
| SEC-M-04 | Gateway sin tests anti-PII en logs | Módulo redact como Oracle |
| SEC-M-05 | Fallback IP `127.0.0.1` en Oracle sin header | Validar en prod |
| SEC-M-06 | SSRF vía URLs riel configurables | Validar esquema/host en Fase 7 |

### Bajo

| ID | Hallazgo | Notas |
|----|----------|-------|
| SEC-B-01 | `/health` público | Aceptado — orquestación |
| SEC-B-02 | Antifraude decline → `402` opaco | Correcto para no filtrar fraude |
| SEC-B-03 | Claves demo en `.env.example` | OK si `.env` no se commitea |
| SEC-B-04 | Sin `cargo audit` en CI | Paso 6.6 |

---

## 4. Frontera Oracle (§9.4) — evaluación

| Control | Estado | Tests |
|---------|--------|-------|
| Aislamiento (frontend nunca llama Oracle) | ✅ | Arquitectura + `frontend/src/api/gateway.ts` |
| `X-API-KEY` obligatorio | ✅ | `auth_security.rs` |
| Allowlist IP/CIDR | ✅ | `security_fail_closed.rs` |
| Rate limit key+IP | ✅ | `rate_limit_integration.rs` |
| Fail closed (sin hold en rechazo) | ✅ | UC-11 suite |
| PAN no persiste en BD Oracle | ✅ | `approved_authorization_persists_token_hash_not_pan` |
| Logs sin PII | ✅ | `logging_no_pii.rs` |
| Antifraude fail closed | ✅ | `antifraud_integration.rs` |
| mTLS | ⏭ Fase 7 | D7 |

---

## 5. PCI simulado (§9.6)

| Requisito PCI (MVP) | Estado |
|---------------------|--------|
| CVV nunca persistido | ✅ No en schemas BD Gateway/Oracle |
| PAN no persiste Gateway | ✅ Corregido idempotency (SEC-6.4-02) |
| PAN no persiste Oracle | ✅ Solo `card_token_hash` |
| CDE aislada (Oracle) | ✅ Frontera HTTP interna |
| Tokenización en memoria | 🔄 Hash demo — mejorar Fase 7 |
| Logs sin PAN/CVV | ✅ Oracle; Gateway parcial |
| 3DS / SCA | ⏭ Post-MVP D8 |

---

## 6. OWASP Top 10 — estado MVP

| ID | Categoría | Estado | Notas |
|----|-----------|--------|-------|
| A01 | Broken Access Control | **Mejorado** | IDOR corregido; Oracle allowlist con caveat XFF |
| A02 | Cryptographic Failures | Parcial | Token PAN débil; API key frontend demo |
| A03 | Injection | ✅ | sqlx parametrizado; Zod frontend |
| A04 | Insecure Design | Aceptado MVP | Fail closed documentado |
| A05 | Security Misconfiguration | Débil | Sin TLS/CORS/headers; bind 0.0.0.0 |
| A06 | Vulnerable Components | No evaluado | CI 6.6 pendiente |
| A07 | Auth Failures | Parcial | Auth presente; timing side-channel |
| A08 | Software Integrity | Parcial | Anchor constraints OK |
| A09 | Logging Failures | Parcial | Oracle fuerte; Gateway mejorable |
| A10 | SSRF | Bajo | URLs riel por env |

---

## 7. Web3 / on-chain

| Control | Estado | Evidencia |
|---------|--------|-----------|
| `PaymentProcessed` sin PII | ✅ | `programs/payment-settlement/src/state.rs` |
| Overflow checked | ✅ | `process-payment-constraints.ts` |
| Signer validation | ✅ | Tests Anchor |
| Zero seed phrase en UI | ✅ | No solicitado en frontend |

---

## 8. Cobertura de tests de seguridad

| Suite | Alcance |
|-------|---------|
| `oracle/tests/security_fail_closed.rs` | UC-11 fail closed |
| `oracle/tests/auth_security.rs` | API key, IP |
| `oracle/tests/rate_limit_integration.rs` | 429 |
| `oracle/tests/logging_no_pii.rs` | PCI logs |
| `crates/api-gateway/tests/security_integration.rs` | IDOR, fingerprint PCI |
| `crates/api-gateway/tests/idempotency_integration.rs` | D9 |
| `crates/api-gateway/tests/error_mapping_integration.rs` | HTTP §6.2 |

---

## 9. Plan de acción post-revisión

| Prioridad | Acción | Fase |
|-----------|--------|------|
| P0 | ~~IDOR transacciones~~ | ✅ 6.4 |
| P0 | ~~Fingerprint sin PAN~~ | ✅ 6.4 |
| P0 | Documentar API key frontend solo demo | ✅ este doc |
| P1 | HMAC token PAN | 7 |
| P1 | Constant-time API keys | 7 |
| P1 | IP de confianza (no XFF cliente) | 7 |
| P2 | CORS + CSP + HSTS | 7 |
| P2 | Errores 500 genéricos | 7 |
| P3 | mTLS Gateway↔Oracle | 7/8 |

---

## 10. Aprobación

| Rol | Criterio | Estado | Fecha |
|-----|----------|--------|-------|
| Dev / Security | Revisión §9 completada | ✅ | 2026-07-26 |
| Dev / Security | Remediaciones P0 aplicadas | ✅ | 2026-07-26 |
| Dev / Security | Tests `security_integration` verdes | ✅ | 2026-07-26 |
| Producto | Aceptación riesgos MVP documentados | ⬜ | |

---

## Referencias

- [Arquitectura.md §9](./Arquitectura.md#9-seguridad)
- [Checklist-QA-Fase-6.md §12 UC-11](./Checklist-QA-Fase-6.md#12-uc-11--control-de-acceso-al-oracle)
- [Plan-de-Implementacion.md §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening)
- [Deuda-Tecnica.md](./Deuda-Tecnica.md)
