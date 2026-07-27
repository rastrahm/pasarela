# Performance Review — Phase 6.5

> Checkout budget ~1–3 s · Bottlenecks · Configured timeouts  
> **Date:** 2026-07-26 · **Status:** Completed

References: [Arquitectura-en.md §4.4](./Arquitectura-en.md#44-liquidación-por-riel) · [Plan §10](./Plan-de-Implementacion-en.md#10-fase-6--integración-qa-y-hardening) · D10 commitment `finalized`

---

## 1. Executive summary

| Scope | Result | Meets 1–3 s |
|-------|--------|-------------|
| Checkout stub (mock Oracle + in-memory settlement) | **p99 < 500 ms** (measured in local CI) | ✅ |
| Cross-service checkout (real Oracle + stub settlement) | Estimated **200–800 ms** without remote PG | ✅ |
| Bank / Binance rail (local or HTTP sim settlement) | **< 1 s** typical | ✅ |
| **Devnet** Solana rail with `finalized` (D10) | **5–60 s** depending on RPC | ⚠️ May exceed 3 s |
| Fallback UC-08 (2+ Oracle authorize) | Multiplies Oracle latency | ⚠️ Under failure load |

**Conclusion:** No obvious bottlenecks on MVP happy path with stubs. The only component that can intentionally exceed 3 s is on-chain Solana with `finalized` commitment (D10). Sequential persistence and multiple writes are post-MVP optimization debt, not Phase 6 gate blockers.

---

## 2. Automated measurements (2026-07-26)

Suite: `crates/api-gateway/tests/performance_integration.rs`

| Test | Budget | Result |
|------|--------|--------|
| `performance_checkout_stub_path_under_budget_per_rail` | p99 ≤ **500 ms** × 3 rails × 5 samples | ✅ PASS (~10 ms/sample in debug) |
| `performance_health_endpoint_under_50ms` | ≤ **50 ms** | ✅ PASS |

```bash
cargo test -p api-gateway --test performance_integration
```

> Note: **debug** build in dev; release would be faster. The 500 ms budget leaves margin for shared CI.

---

## 3. Latency budget by phase (UC-01)

Sequential flow in `crates/api-gateway/src/services/checkout.rs`:

```text
select_rail → save_tx → audit → Oracle authorize → save_tx → audit → settle → save_tx → insert_settlement → audit
```

| Phase | Component | Stub/MVP | Simulated production | Budget |
|-------|-----------|----------|----------------------|--------|
| 1 | Rail Switcher | In-process | In-process | < 5 ms |
| 2 | Gateway persistence (×4 writes) | In-memory / local PG | Remote PG | 5–50 ms / 20–150 ms |
| 3 | **Oracle authorize** | HTTP mock ~1 ms | Oracle+PG+antifraud | **100–400 ms** |
| 3a | Luhn + tokenization | In-process | In-process | < 5 ms |
| 3b | Antifraud HTTP | Mock approve | `antifraud/` round-trip | 10–80 ms |
| 3c | Rail funds evaluation | Mock provider | Binance/RPC HTTP | 20–200 ms |
| 3d | Hold persist (Oracle PG) | Test PG | Dedicated PG | 10–50 ms |
| 4 | **Settlement** | In-memory stub | See §4 by rail | **1 ms – 60 s** |
| 5 | Gateway audit log | In-memory | PG | 5–20 ms |

**Happy path total (bank/Binance stub):** ~150–800 ms with local stack.  
**Plan target 1–3 s:** met with margin.

---

## 4. Latency by settlement rail

| Rail | Default dev adapter | Real settlement | Bottlenecks |
|------|---------------------|-----------------|-------------|
| **TraditionalBank** | `TraditionalBankAdapter` — in-process ISO 20022 | Same (XML generation) | None MVP |
| **BinanceCex** | `BinanceCexAdapter::default()` — in-memory | `HttpBinanceSpotClient` → `binance-sim/` | HTTP + spread calc |
| **SolanaWallet** | `SolanaWalletAdapter::default()` — in-memory mock | `RpcSolanaSettlementClient` + **`finalized`** | RPC + D10 confirmation |

### Solana — D10 exception

| Config | Default | Impact |
|--------|---------|--------|
| `SOLANA_CONFIRM_TIMEOUT_SECS` | **60 s** | Ceiling before timeout error |
| Commitment | **`finalized`** | 15–30 s mainnet; 2–15 s devnet typical |

Architecture §851: *"Implies higher latency in exchange for irreversibility"*.  
**Not a bug** — exceeding 3 s on real Solana is expected until Phase 7 (async UX / polling).

---

## 5. Configured timeouts

| Service | Variable | Default | Checkout effect |
|---------|----------|---------|-----------------|
| Gateway → Oracle | `ORACLE_TIMEOUT_SECS` | 5 s | HTTP authorize/release cap |
| Oracle → rail | `ORACLE_RAIL_TIMEOUT_SECS` | 5 s | Binance/RPC balance |
| Oracle → antifraud | `ANTIFRAUD_TIMEOUT_SECS` | 5 s | Fail closed on timeout |
| Binance settlement | `BINANCE_*` client timeout | 5 s | Spot debit |
| Solana settlement | `SOLANA_CONFIRM_TIMEOUT_SECS` | 60 s | Wait for `finalized` |

**Theoretical worst-case chain (sequential):** ~5 s Oracle + ~60 s Solana → user sees UX timeout first if frontend does not extend timeout.

---

## 6. Identified bottlenecks

### Accepted MVP

| ID | Bottleneck | Justification |
|----|------------|---------------|
| PERF-M-01 | **100% sequential** flow (no parallelism) | UC-01 simplicity; authorize must precede settle |
| PERF-M-02 | **4–5 Gateway writes** per checkout | Audit trail; batch in Phase 7 |
| PERF-M-03 | Solana **`finalized`** > 3 s | D10 irreversibility |
| PERF-M-04 | Fallback = **N × Oracle authorize** | D3 design |

### Recommended improvements (Phase 7+)

| ID | Improvement | Estimated impact |
|----|-------------|------------------|
| PERF-R-01 | Async Solana response (`202 Pending` + UC-09 polling) | UX under on-chain latency |
| PERF-R-02 | Batch audit / write-behind Gateway persistence | −30–50 ms |
| PERF-R-03 | PG connection pool tuning (Gateway + Oracle) | −20 ms under load |
| PERF-R-04 | Rail balance cache (short TTL) in Oracle | −50–100 ms per authorize |
| PERF-R-05 | Differentiated `ORACLE_RAIL_TIMEOUT_SECS` per rail | Fail fast Solana vs bank |
| PERF-R-06 | k6/vegeta load test (Plan 7.12) | Validate rate limit |

---

## 7. Frontend

| Aspect | Status | Notes |
|--------|--------|-------|
| Zod validation pre-request | ✅ | Avoids invalid round-trip |
| Single checkout `fetch` | ✅ | No waterfall |
| Explicit `fetch` timeout | ⬜ | Uses browser default; consider AbortSignal 30 s |
| Loading state | ✅ | `Procesando pago…` disables UI |
| Bundle size | Not evaluated | Vite tree-shaking OK MVP |

---

## 8. Manual verification (real stack)

See [Doc/Runbook-Desarrollo-en.md](./Runbook-Desarrollo-en.md) to start the stack and run manual benches.

```bash
# Health
curl -w "\nTOTAL: %{time_total}s\n" -s -o /dev/null http://127.0.0.1:8080/health

# Checkout (replace API_KEY and Idempotency-Key)
time curl -s -X POST http://127.0.0.1:8080/api/v1/checkout \
  -H "Authorization: Bearer $API_KEY" \
  -H "Idempotency-Key: bench-$(uuidgen)" \
  -H "Content-Type: application/json" \
  -d @scripts/fixtures/checkout-bank.json
```

Record per rail: bank, Binance (with `binance-sim`), Solana devnet.

---

## 9. Phase 6.5 gate criteria

| Criterion | Status |
|-----------|--------|
| 1–3 s budget documented by phase | ✅ |
| No obvious bottlenecks on stub path | ✅ performance tests |
| Solana D10 documented as exception | ✅ |
| Timeouts inventoried | ✅ §5 |
| Phase 7 improvement plan | ✅ §6 |

---

## 10. Approval

| Role | Criterion | Status | Date |
|------|-----------|--------|------|
| Dev | `performance_integration` tests green | ✅ | 2026-07-26 |
| Dev | Review document completed | ✅ | 2026-07-26 |
| Product | Acceptance of Solana > 3 s exception | ⬜ | |

---

## References

- [Revision-Seguridad-Fase-6-en.md](./Revision-Seguridad-Fase-6-en.md)
- [Checklist-QA-Fase-6-en.md](./Checklist-QA-Fase-6-en.md)
- [Deuda-Tecnica-en.md](./Deuda-Tecnica-en.md)
- `crates/api-gateway/tests/performance_integration.rs`
