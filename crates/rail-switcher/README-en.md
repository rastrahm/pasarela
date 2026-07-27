# rail-switcher

Rail decision engine — **Phase 1** (Strategy / fallback D3).

> Pure library: chooses among `TraditionalBank`, `BinanceCex`, and `SolanaWallet` based on preference, availability, cost, and configuration. **No** HTTP or Oracle calls; it receives pre-evaluated snapshots.

## Responsibility

Implements business rule **D3** (Architecture §4.3):

1. Explicit checkout preference (UC-02)
2. Merchant default when no explicit preference
3. Fallback order: bank → CEX → Solana
4. Rejection when no rail is viable (`RailError`)

## Main API

| Type / method | Role |
|---------------|------|
| `RailSwitcher` | Stateless engine (`Default`, `Copy`) |
| `RailSwitcher::select` | Selection with priority-based fallback |
| `RailSwitcher::select_lowest_cost` | Tie-break by effective cost (`fee + spread`) |
| `RailSelectionInput` | Amount, currency, preferences, configs, availability |
| `RailConfig` | `RAIL_CONFIG`: enabled, priority, fees |
| `RailPreference` | `RAIL_PREFERENCE`: preferred, merchant default, fallback |
| `RailAvailability` | Snapshot: operational + funds_sufficient |

## Algorithm (`select`)

```
1. Build candidates: preferred_rail → merchant_default → default order
2. For each candidate:
   - Enabled in RailConfig?
   - Operational with sufficient funds (RailAvailability)?
   - Amount > 0?
3. If preferred fails and fallback_enabled == false → immediate error
4. If all fail → RailError::NoRailAvailable
```

Default order when no preference is set:

`TraditionalBank` → `BinanceCex` → `SolanaWallet`

## Usage

```rust
use domain::{Amount, Currency, FundingType};
use rail_switcher::{
    RailAvailability, RailConfig, RailPreference, RailSelectionInput, RailSwitcher,
};
use rust_decimal::Decimal;
use std::str::FromStr;

let switcher = RailSwitcher;
let currency = Currency::from_str("USD").unwrap();

let preference = RailPreference {
    preferred_rail: Some(FundingType::BinanceCex),
    merchant_default: None,
    fallback_enabled: true,
};

let configs = vec![/* RailConfig per rail */];
let availability = vec![/* RailAvailability from Oracle snapshot */];

let input = RailSelectionInput {
    amount: Amount::from_units(100),
    currency: &currency,
    preference: &preference,
    configs: &configs,
    availability: &availability,
};

let rail = switcher.select(&input)?;
```

## Consumer

| Crate | Usage |
|-------|-------|
| [`api-gateway`](../api-gateway/) | `services/rails.rs` — rail selection at checkout |

The Gateway builds `RailContext` (configs + availability) and delegates to `RailSwitcher::select`.

## Dependencies

- [`domain`](../domain/) — `FundingType`, `Amount`, `Currency`, `RailError`

> The Oracle does **not** depend on this crate (workspace rule in root `Cargo.toml`).

## Tests

```bash
cargo test -p rail-switcher
```

Covered cases: explicit preference, merchant default, fallback on insufficient funds, fallback disabled, disabled rail in config, `select_lowest_cost`.

## References

- [Doc/Arquitectura-en.md](../../Doc/Arquitectura-en.md) — decision D3, Rail Switcher diagram
- [Doc/Casos-de-Uso-ER-Flujos-en.md](../../Doc/Casos-de-Uso-ER-Flujos-en.md) — UC-02, UC-08
- [domain/README-en.md](../domain/README-en.md) — shared types
