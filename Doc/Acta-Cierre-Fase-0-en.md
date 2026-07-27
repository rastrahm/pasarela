# Phase Closure Record — Phase 0 (Planning)

> Gate **0.8** · Documentation validation and approval  
> Date: **2026-07-25**  
> Project: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaration

**Phase 0 — Planning** of the multi-rail payment processor is declared **closed**, with authorization to begin **Phase 1 — Domain and rail abstraction**.

Planning defines the MVP scope, microservices architecture, use cases, data model, operational flows, design decisions D1–D12, and the roadmap to production.

---

## 2. Verified deliverables

| Deliverable | Location | Status |
|-------------|----------|--------|
| Context and MVP scope | [Contexto General-en.md](./Contexto%20General-en.md) | ✅ |
| System architecture | [Arquitectura-en.md](./Arquitectura-en.md) | ✅ |
| Use cases, ER, and flows | [Casos-de-Uso-ER-Flujos-en.md](./Casos-de-Uso-ER-Flujos-en.md) | ✅ |
| Implementation plan | [Plan-de-Implementacion-en.md](./Plan-de-Implementacion-en.md) | ✅ |
| Decisions D1–D12 | [Arquitectura §12](./Arquitectura-en.md#12-decisiones-de-diseño--resueltas-fase-0) | ✅ |
| Oracle skeleton (Axum, auth, Luhn) | `oracle/` | ✅ (15 green tests) |
| Code directives | `*.cursorrules` | ✅ |

---

## 3. Document coherence checklist

Cross-review between Architecture, Use Cases, and Plan:

| # | Criterion | Result |
|---|-----------|--------|
| 1 | Three rails defined (TraditionalBank, BinanceCex, SolanaWallet) | ✅ Consistent across all 3 docs |
| 2 | Independent Oracle in `oracle/` (monorepo D5) | ✅ Consistent |
| 3 | External antifraud `antifraud/` (D11) | ✅ Architecture + UC-12 + flows |
| 4 | Axum framework (D1) | ✅ Consistent |
| 5 | Automatic fallback by priority (D3) | ✅ Rail Switcher + UC-08 |
| 6 | Configurable Binance spread (D4) | ✅ Env + UC-04 |
| 7 | PAN token in memory (D6) | ✅ Oracle + UC-03 |
| 8 | mTLS deferred to Phase 7/8 (D7) | ✅ Consistent |
| 9 | 3DS out of MVP (D8) | ✅ Consistent |
| 10 | Idempotency-Key in Phase 4 (D9) | ✅ UC-01 + Gateway |
| 11 | Solana commitment `finalized` (D10) | ✅ UC-07 + §4.4.3 |
| 12 | Merchant API key (D12) | ✅ MERCHANT ER + UC-01 |
| 13 | Boundary: Frontend never → Oracle | ✅ Consistent |
| 14 | Zero PII on-chain | ✅ PaymentProcessed |
| 15 | Sequential phases with gates | ✅ Plan §2 and §4–12 |

**Result:** documentation is **coherent** with itself. No blocking contradictions were detected.

---

## 4. Recorded design decisions (summary)

See full table in [Arquitectura §12](./Arquitectura-en.md#12-decisiones-de-diseño--resueltas-fase-0).

| ID | Resolution |
|----|------------|
| D1 | Axum (Gateway + Oracle + Antifraud) |
| D2 | Local validator (dev) + devnet (CI) |
| D3 | Automatic fallback by priority |
| D4 | Configurable `BINANCE_SPREAD_BUFFER_PCT` |
| D5 | Monorepo |
| D6 | Token hash in memory; PAN discarded |
| D7 | mTLS in Phase 7/8 |
| D8 | 3DS post-MVP |
| D9 | Idempotency-Key in Phase 4 |
| D10 | Commitment `finalized` |
| D11 | Simulated `antifraud/` service |
| D12 | Merchant API key in Phase 4 |

---

## 5. Agreed MVP scope

### Included

- Payment processor with fictitious card (Luhn)
- Three interchangeable settlement rails
- Isolated authorization Oracle
- Simulated antifraud service
- Anchor Solana program (devnet/local)
- API Gateway with orchestration
- Demo checkout frontend
- Fail-closed security, no PII on-chain

### Excluded (post-MVP)

- 3-D Secure / PSD2 SCA
- Real acquirer integration (Stripe, Adyen)
- HSM token vault / PCI level 1
- Real KYC/AML
- Solana mainnet without audit
- mTLS (until staging)

---

## 6. Pre-Phase 1 technical verification

| Verification | Command / action | Result |
|--------------|------------------|--------|
| Oracle compiles and tests pass | `cd oracle && cargo test` | ✅ 15 tests OK |
| Pasarela workspace | — | ⬜ Pending (Phase 1) |
| Anchor program | — | ⬜ Pending (Phase 3) |

---

## 7. Authorization

| Role | Action | Date |
|------|--------|------|
| Stakeholder / Product | Phase 0 gate approval → Phase 1 start | 2026-07-25 |
| Architecture | Documentation validated (checklist §3) | 2026-07-25 |

**Authorized next step:** Phase 1 — create Cargo workspace in `pasarela/` with crates `domain` and `rail-switcher`.

---

## References

- [Plan-de-Implementacion-en.md §4](./Plan-de-Implementacion-en.md#4-fase-0--planificación-fase-actual)
- [Arquitectura-en.md](./Arquitectura-en.md)
- [Casos-de-Uso-ER-Flujos-en.md](./Casos-de-Uso-ER-Flujos-en.md)
