# Phase Closure Record — Phase 1 (Domain and rail abstraction)

> Gate **1.9** · Technical verification and domain closure  
> Date: **2026-07-25**  
> Project: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaration

**Phase 1 — Domain and rail abstraction** is declared **closed**, with the shared domain layer implemented in Rust and the test suite verified.

Advancement toward **Phase 3 (Solana/Anchor)** and/or formal closure of **Phase 2 (Oracle)** gate is authorized, per team priority.

---

## 2. Verified deliverables

| Deliverable | Location | Status |
|-------------|----------|--------|
| Cargo workspace (root) | `Cargo.toml` | ✅ |
| Crate `domain` | `crates/domain/` | ✅ |
| Crate `rail-switcher` | `crates/rail-switcher/` | ✅ |
| Traits `PaymentProcessor`, `LiquidityEngine` | `crates/domain/src/traits.rs` | ✅ |
| Typed errors | `crates/domain/src/error.rs` | ✅ |
| Rail decision engine | `crates/rail-switcher/src/switcher.rs` | ✅ |

---

## 3. Step checklist (Plan §5)

| # | Step | Result |
|---|------|--------|
| 1.1 | Root Cargo workspace | ✅ *(also includes Phase 2 members — see §6)* |
| 1.2 | `domain` crate with newtypes | ✅ |
| 1.3 | Core traits | ✅ |
| 1.4 | Domain structs/enums | ✅ |
| 1.5 | `thiserror` errors | ✅ |
| 1.6 | `rail-switcher` crate | ✅ |
| 1.7 | TDD unit tests | ✅ (12 tests) |
| 1.8 | `///` documentation on public API | ✅ |
| 1.9 | `cargo test` + `cargo clippy` | ✅ |

---

## 4. Acceptance criteria (gate)

| Criterion | Verification | Result |
|-----------|--------------|--------|
| Green `cargo test` in `domain` and `rail-switcher` | `cargo test -p domain -p rail-switcher` | ✅ 12/12 |
| Rail Switcher: explicit preference | `selects_explicit_preference_when_viable` | ✅ |
| Rail Switcher: merchant default | `uses_merchant_default_without_explicit_preference` | ✅ |
| Rail Switcher: automatic fallback (D3) | `falls_back_when_preferred_rail_lacks_funds` | ✅ |
| Rail Switcher: reject with no viable rail | `rejects_when_no_rail_is_viable` | ✅ |
| Rail Switcher: fallback disabled | `rejects_when_fallback_disabled_and_preferred_fails` | ✅ |
| Rail Switcher: disabled rails | `skips_disabled_rails_in_config` | ✅ |
| Cost-based tiebreaker | `select_lowest_cost_picks_cheapest_viable_rail` | ✅ |
| No HTTP/DB/Solana in `domain` | `Cargo.toml` review | ✅ |
| `cargo clippy` without warnings | `cargo clippy -p domain -p rail-switcher -- -D warnings` | ✅ |
| Stakeholder confirmation | Formal gate | ✅ 2026-07-25 |

---

## 5. Technical verification executed

```bash
# Phase 1 — domain
cargo test -p domain -p rail-switcher
cargo clippy -p domain -p rail-switcher -- -D warnings
```

| Crate | Tests | Clippy |
|-------|-------|--------|
| `domain` | 5 passed | ✅ clean |
| `rail-switcher` | 7 passed | ✅ clean |

---

## 6. Documented deviations

| Item | Original plan | Current state | Impact |
|------|---------------|---------------|--------|
| Workspace members | Pasarela crates only; **do not** include `oracle/` | Workspace includes `oracle/`, `antifraud/`, `binance-sim/`, `oracle-client` | Low — enables unified CI; Oracle remains logically isolated |
| Phase order | Phase 1 before Phase 2 | Phase 2 implemented in parallel (prior branch) | None for domain — independent crates |

---

## 7. Types and exposed API

### `domain`

- **Newtypes:** `TransactionId`, `MerchantId`, `Amount`, `Currency`, `HoldId`, `CardNumber`
- **Enums:** `FundingType`, `TransactionStatus`
- **Structs:** `PaymentRequest`, `PaymentResponse`, `CardPayload`, `FundStatus`, `SettlementReceipt`
- **Errors:** `PaymentError`, `LiquidityError`, `RailError`
- **Traits:** `PaymentProcessor`, `LiquidityEngine`

### `rail-switcher`

- **Config:** `RailConfig`, `RailPreference`, `RailAvailability`, `RailSelectionInput`
- **Engine:** `RailSwitcher::select()`, `RailSwitcher::select_lowest_cost()`

---

## 8. Authorization

| Role | Action | Date |
|------|--------|------|
| Technical verification (agent/dev) | Gate 1.9 — green tests and clippy | 2026-07-25 |
| Stakeholder / Product | Phase 1 gate approval → Phase 3 / Phase 4 | 2026-07-25 |

**Suggested next step:** Phase 3 (Anchor program) or close Phase 2 gate (stakeholder confirmation).

---

## References

- [Plan-de-Implementacion-en.md §5](./Plan-de-Implementacion-en.md#5-fase-1--dominio-y-abstracción-de-rieles)
- [Arquitectura-en.md §5](./Arquitectura-en.md#5-capa-de-dominio-fase-1)
- [Casos-de-Uso-ER-Flujos-en.md §4.3](./Casos-de-Uso-ER-Flujos-en.md#43-flujo-de-decisión-del-rail-switcher)
- [Acta-Cierre-Fase-0-en.md](./Acta-Cierre-Fase-0-en.md)
