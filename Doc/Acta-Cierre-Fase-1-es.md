# Acta de Cierre — Fase 1 (Dominio y abstracción de rieles)

> Gate **1.9** · Verificación técnica y cierre de dominio  
> Fecha: **2026-07-25**  
> Proyecto: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaración

Se declara **cerrada la Fase 1 — Dominio y abstracción de rieles**, con la capa de dominio compartida implementada en Rust y suite de tests verificada.

Queda autorizado avanzar hacia **Fase 3 (Solana/Anchor)** y/o consolidar el gate formal de **Fase 2 (Oracle)**, según prioridad del equipo.

---

## 2. Entregables verificados

| Entregable | Ubicación | Estado |
|------------|-----------|--------|
| Workspace Cargo (raíz) | `Cargo.toml` | ✅ |
| Crate `domain` | `crates/domain/` | ✅ |
| Crate `rail-switcher` | `crates/rail-switcher/` | ✅ |
| Traits `PaymentProcessor`, `LiquidityEngine` | `crates/domain/src/traits.rs` | ✅ |
| Errores tipados | `crates/domain/src/error.rs` | ✅ |
| Motor de decisión de riel | `crates/rail-switcher/src/switcher.rs` | ✅ |

---

## 3. Checklist de pasos (Plan §5)

| # | Paso | Resultado |
|---|------|-----------|
| 1.1 | Workspace Cargo en raíz | ✅ *(incluye también miembros de Fase 2 — ver §6)* |
| 1.2 | Crate `domain` con newtypes | ✅ |
| 1.3 | Traits principales | ✅ |
| 1.4 | Structs/enums de dominio | ✅ |
| 1.5 | Errores `thiserror` | ✅ |
| 1.6 | Crate `rail-switcher` | ✅ |
| 1.7 | Tests unitarios TDD | ✅ (12 tests) |
| 1.8 | Documentación `///` en API pública | ✅ |
| 1.9 | `cargo test` + `cargo clippy` | ✅ |

---

## 4. Criterios de aceptación (gate)

| Criterio | Verificación | Resultado |
|----------|--------------|-----------|
| `cargo test` verde en `domain` y `rail-switcher` | `cargo test -p domain -p rail-switcher` | ✅ 12/12 |
| Rail Switcher: preferencia explícita | `selects_explicit_preference_when_viable` | ✅ |
| Rail Switcher: default comercio | `uses_merchant_default_without_explicit_preference` | ✅ |
| Rail Switcher: fallback automático (D3) | `falls_back_when_preferred_rail_lacks_funds` | ✅ |
| Rail Switcher: rechazo sin riel viable | `rejects_when_no_rail_is_viable` | ✅ |
| Rail Switcher: fallback deshabilitado | `rejects_when_fallback_disabled_and_preferred_fails` | ✅ |
| Rail Switcher: rieles deshabilitados | `skips_disabled_rails_in_config` | ✅ |
| Desempate por costo | `select_lowest_cost_picks_cheapest_viable_rail` | ✅ |
| Sin HTTP/DB/Solana en `domain` | Revisión `Cargo.toml` | ✅ |
| `cargo clippy` sin warnings | `cargo clippy -p domain -p rail-switcher -- -D warnings` | ✅ |
| Confirmación stakeholder | Gate formal | ✅ 2026-07-25 |

---

## 5. Verificación técnica ejecutada

```bash
# Fase 1 — dominio
cargo test -p domain -p rail-switcher
cargo clippy -p domain -p rail-switcher -- -D warnings
```

| Crate | Tests | Clippy |
|-------|-------|--------|
| `domain` | 5 passed | ✅ limpio |
| `rail-switcher` | 7 passed | ✅ limpio |

---

## 6. Desviaciones documentadas

| Ítem | Plan original | Estado actual | Impacto |
|------|---------------|---------------|---------|
| Workspace members | Solo crates pasarela; **no** incluir `oracle/` | Workspace incluye `oracle/`, `antifraud/`, `binance-sim/`, `oracle-client` | Bajo — facilita CI unificada; Oracle sigue siendo servicio lógicamente aislado |
| Orden de fases | Fase 1 antes de Fase 2 | Fase 2 implementada en paralelo (rama previa) | Ninguno en dominio — crates independientes |

---

## 7. Tipos y API expuesta

### `domain`

- **Newtypes:** `TransactionId`, `MerchantId`, `Amount`, `Currency`, `HoldId`, `CardNumber`
- **Enums:** `FundingType`, `TransactionStatus`
- **Structs:** `PaymentRequest`, `PaymentResponse`, `CardPayload`, `FundStatus`, `SettlementReceipt`
- **Errores:** `PaymentError`, `LiquidityError`, `RailError`
- **Traits:** `PaymentProcessor`, `LiquidityEngine`

### `rail-switcher`

- **Config:** `RailConfig`, `RailPreference`, `RailAvailability`, `RailSelectionInput`
- **Motor:** `RailSwitcher::select()`, `RailSwitcher::select_lowest_cost()`

---

## 8. Autorización

| Rol | Acción | Fecha |
|-----|--------|-------|
| Verificación técnica (agente/dev) | Gate 1.9 — tests y clippy verdes | 2026-07-25 |
| Stakeholder / Producto | Aprobación gate Fase 1 → Fase 3 / Fase 4 | 2026-07-25 |

**Próximo paso sugerido:** Fase 3 (programa Anchor) o cerrar gate Fase 2 (confirmación stakeholder).

---

## Referencias

- [Plan-de-Implementacion.md §5](./Plan-de-Implementacion.md#5-fase-1--dominio-y-abstracción-de-rieles)
- [Arquitectura.md §5](./Arquitectura.md#5-capa-de-dominio-fase-1)
- [Casos-de-Uso-ER-Flujos.md §4.3](./Casos-de-Uso-ER-Flujos.md#43-flujo-de-decisión-del-rail-switcher)
- [Acta-Cierre-Fase-0.md](./Acta-Cierre-Fase-0.md)
