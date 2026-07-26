# Revisión de rendimiento — Fase 6.5

> Presupuesto checkout ~1–3 s · Cuellos de botella · Timeouts configurados  
> **Fecha:** 2026-07-26 · **Estado:** Completada

Referencias: [Arquitectura.md §4.4](./Arquitectura.md#44-liquidación-por-riel) · [Plan §10](./Plan-de-Implementacion.md#10-fase-6--integración-qa-y-hardening) · D10 commitment `finalized`

---

## 1. Resumen ejecutivo

| Ámbito | Resultado | Cumple 1–3 s |
|--------|-----------|--------------|
| Checkout stub (mock Oracle + settlement in-memory) | **p99 < 500 ms** (medido en CI local) | ✅ |
| Checkout cross-service (Oracle real + stub settlement) | Estimado **200–800 ms** sin PG remoto | ✅ |
| Riel banco / Binance (settlement local o HTTP sim) | **< 1 s** típico | ✅ |
| Riel Solana **devnet** con `finalized` (D10) | **5–60 s** según RPC | ⚠️ Puede exceder 3 s |
| Fallback UC-08 (2+ authorize Oracle) | Multiplica latencia Oracle | ⚠️ Bajo carga de fallos |

**Conclusión:** No hay cuellos obvios en el camino feliz MVP con stubs. El único componente que puede superar 3 s de forma **intencional** es Solana on-chain con commitment `finalized` (D10). Persistencia secuencial y múltiples writes son deuda de optimización post-MVP, no bloqueantes para gate Fase 6.

---

## 2. Mediciones automatizadas (2026-07-26)

Suite: `crates/api-gateway/tests/performance_integration.rs`

| Test | Presupuesto | Resultado |
|------|-------------|-----------|
| `performance_checkout_stub_path_under_budget_per_rail` | p99 ≤ **500 ms** × 3 rieles × 5 muestras | ✅ PASS (~10 ms/muestra en debug) |
| `performance_health_endpoint_under_50ms` | ≤ **50 ms** | ✅ PASS |

```bash
cargo test -p api-gateway --test performance_integration
```

> Nota: build **debug** en dev; release sería más rápido. El presupuesto 500 ms deja margen para CI compartida.

---

## 3. Presupuesto de latencia por fase (UC-01)

Flujo secuencial en `crates/api-gateway/src/services/checkout.rs`:

```text
select_rail → save_tx → audit → Oracle authorize → save_tx → audit → settle → save_tx → insert_settlement → audit
```

| Fase | Componente | Stub/MVP | Producción simulada | Presupuesto |
|------|------------|----------|---------------------|-------------|
| 1 | Rail Switcher | In-process | In-process | < 5 ms |
| 2 | Persistencia Gateway (×4 writes) | In-memory / PG local | PG remoto | 5–50 ms / 20–150 ms |
| 3 | **Oracle authorize** | HTTP mock ~1 ms | Oracle+PG+antifrraud | **100–400 ms** |
| 3a | Luhn + tokenización | In-process | In-process | < 5 ms |
| 3b | Antifraude HTTP | Mock approve | `antifraud/` round-trip | 10–80 ms |
| 3c | Evaluación fondos riel | Mock provider | Binance/RPC HTTP | 20–200 ms |
| 3d | Hold persist (PG Oracle) | Test PG | PG dedicado | 10–50 ms |
| 4 | **Settlement** | In-memory stub | Ver §4 por riel | **1 ms – 60 s** |
| 5 | Audit log Gateway | In-memory | PG | 5–20 ms |

**Total camino feliz (banco/Binance stub):** ~150–800 ms con stack local.  
**Target plan 1–3 s:** cumplido con margen.

---

## 4. Latencia por riel de liquidación

| Riel | Adaptador default dev | Settlement real | Cuellos |
|------|----------------------|-----------------|---------|
| **TraditionalBank** | `TraditionalBankAdapter` — ISO 20022 in-process | Mismo (generación XML) | Ninguno MVP |
| **BinanceCex** | `BinanceCexAdapter::default()` — in-memory | `HttpBinanceSpotClient` → `binance-sim/` | HTTP + spread calc |
| **SolanaWallet** | `SolanaWalletAdapter::default()` — in-memory mock | `RpcSolanaSettlementClient` + **`finalized`** | RPC + confirmación D10 |

### Solana — excepción D10

| Config | Default | Impacto |
|--------|---------|---------|
| `SOLANA_CONFIRM_TIMEOUT_SECS` | **60 s** | Techo antes de error timeout |
| Commitment | **`finalized`** | 15–30 s mainnet; 2–15 s devnet típico |

Arquitectura §851: *"Implica mayor latencia a cambio de irreversibilidad"*.  
**No es bug** — superar 3 s en Solana real es esperado hasta Fase 7 (UX async / polling).

---

## 5. Timeouts configurados

| Servicio | Variable | Default | Efecto en checkout |
|----------|----------|---------|-------------------|
| Gateway → Oracle | `ORACLE_TIMEOUT_SECS` | 5 s | Tope HTTP authorize/release |
| Oracle → riel | `ORACLE_RAIL_TIMEOUT_SECS` | 5 s | Balance Binance/RPC |
| Oracle → antifraude | `ANTIFRAUD_TIMEOUT_SECS` | 5 s | Fail closed si timeout |
| Settlement Binance | `BINANCE_*` client timeout | 5 s | Débito Spot |
| Settlement Solana | `SOLANA_CONFIRM_TIMEOUT_SECS` | 60 s | Espera `finalized` |

**Cadena peor caso teórica (secuencial):** ~5 s Oracle + ~60 s Solana → usuario ve timeout UX antes si frontend no extiende timeout.

---

## 6. Cuellos de botella identificados

### Aceptados MVP

| ID | Cuellos | Justificación |
|----|---------|---------------|
| PERF-M-01 | Flujo **100 % secuencial** (sin paralelismo) | Simplicidad UC-01; authorize debe preceder settle |
| PERF-M-02 | **4–5 writes** Gateway por checkout | Audit trail; batch en Fase 7 |
| PERF-M-03 | Solana **`finalized`** > 3 s | D10 irreversibilidad |
| PERF-M-04 | Fallback = **N × authorize** Oracle | D3 diseño |

### Mejoras recomendadas (Fase 7+)

| ID | Mejora | Impacto estimado |
|----|--------|------------------|
| PERF-R-01 | Respuesta async Solana (`202 Pending` + polling UC-09) | UX bajo latencia on-chain |
| PERF-R-02 | Batch audit / write-behind persistencia Gateway | −30–50 ms |
| PERF-R-03 | Connection pool tuning PG (Gateway + Oracle) | −20 ms bajo carga |
| PERF-R-04 | Cache saldo riel (TTL corto) en Oracle | −50–100 ms por authorize |
| PERF-R-05 | `ORACLE_RAIL_TIMEOUT_SECS` diferenciado por riel | Fail fast Solana vs banco |
| PERF-R-06 | Prueba de carga k6/vegeta (Plan 7.12) | Validar rate limit |

---

## 7. Frontend

| Aspecto | Estado | Notas |
|---------|--------|-------|
| Validación Zod pre-red | ✅ | Evita round-trip inválido |
| Un solo `fetch` checkout | ✅ | Sin waterfall |
| Timeout explícito `fetch` | ⬜ | Usa default browser; considerar AbortSignal 30 s |
| Loading state | ✅ | `Procesando pago…` deshabilita UI |
| Bundle size | No evaluado | Vite tree-shaking OK MVP |

---

## 8. Verificación manual (stack real)

Ver [Doc/Runbook-Desarrollo.md](./Runbook-Desarrollo.md) para levantar el stack y ejecutar benches manuales.

```bash
# Health
curl -w "\nTOTAL: %{time_total}s\n" -s -o /dev/null http://127.0.0.1:8080/health

# Checkout (sustituir API_KEY e Idempotency-Key)
time curl -s -X POST http://127.0.0.1:8080/api/v1/checkout \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: bench-$(uuidgen)" \
  -H "Content-Type: application/json" \
  -d @scripts/fixtures/checkout-bank.json
```

Registrar por riel: banco, Binance (con `binance-sim`), Solana devnet.

---

## 9. Criterios gate Fase 6.5

| Criterio | Estado |
|----------|--------|
| Presupuesto 1–3 s documentado por fase | ✅ |
| Sin cuellos obvios en stub path | ✅ tests performance |
| Solana D10 documentado como excepción | ✅ |
| Timeouts inventariados | ✅ §5 |
| Plan mejora Fase 7 | ✅ §6 |

---

## 10. Aprobación

| Rol | Criterio | Estado | Fecha |
|-----|----------|--------|-------|
| Dev | Tests `performance_integration` verdes | ✅ | 2026-07-26 |
| Dev | Documento revisión completado | ✅ | 2026-07-26 |
| Producto | Aceptación excepción Solana > 3 s | ⬜ | |

---

## Referencias

- [Revision-Seguridad-Fase-6.md](./Revision-Seguridad-Fase-6.md)
- [Checklist-QA-Fase-6.md](./Checklist-QA-Fase-6.md)
- [Deuda-Tecnica.md](./Deuda-Tecnica.md)
- `crates/api-gateway/tests/performance_integration.rs`
