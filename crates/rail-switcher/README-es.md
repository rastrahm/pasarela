# rail-switcher

Motor de decisión de riel — **Fase 1** (Strategy / fallback D3).

> Librería pura: elige entre `TraditionalBank`, `BinanceCex` y `SolanaWallet` según preferencia, disponibilidad, costo y configuración. **Sin** HTTP ni consultas al Oracle; recibe snapshots ya evaluados.

## Responsabilidad

Implementa la regla de negocio **D3** (Arquitectura §4.3):

1. Preferencia explícita del checkout (UC-02)
2. Default del comercio si no hay preferencia
3. Orden de fallback: banco → CEX → Solana
4. Rechazo si ningún riel es viable (`RailError`)

## API principal

| Tipo / método | Rol |
|---------------|-----|
| `RailSwitcher` | Motor stateless (`Default`, `Copy`) |
| `RailSwitcher::select` | Selección con fallback por prioridad |
| `RailSwitcher::select_lowest_cost` | Desempate por costo efectivo (`fee + spread`) |
| `RailSelectionInput` | Monto, moneda, preferencias, configs y disponibilidad |
| `RailConfig` | `RAIL_CONFIG`: enabled, priority, fees |
| `RailPreference` | `RAIL_PREFERENCE`: preferred, merchant default, fallback |
| `RailAvailability` | Snapshot: operational + funds_sufficient |

## Algoritmo (`select`)

```
1. Armar candidatos: preferred_rail → merchant_default → orden por defecto
2. Para cada candidato:
   - ¿Habilitado en RailConfig?
   - ¿Operational y con fondos (RailAvailability)?
   - ¿Monto > 0?
3. Si falla el preferido y fallback_enabled == false → error inmediato
4. Si todos fallan → RailError::NoRailAvailable
```

Orden por defecto cuando no hay preferencia:

`TraditionalBank` → `BinanceCex` → `SolanaWallet`

## Uso

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

let configs = vec![/* RailConfig por riel */];
let availability = vec![/* RailAvailability consultado vía Oracle */];

let input = RailSelectionInput {
    amount: Amount::from_units(100),
    currency: &currency,
    preference: &preference,
    configs: &configs,
    availability: &availability,
};

let rail = switcher.select(&input)?;
```

## Consumidor

| Crate | Uso |
|-------|-----|
| [`api-gateway`](../api-gateway/) | `services/rails.rs` — selección en checkout |

El Gateway construye `RailContext` (configs + availability) y delega en `RailSwitcher::select`.

## Dependencias

- [`domain`](../domain/) — `FundingType`, `Amount`, `Currency`, `RailError`

> El Oracle **no** depende de este crate (regla de workspace en `Cargo.toml` raíz).

## Tests

```bash
cargo test -p rail-switcher
```

Casos cubiertos: preferencia explícita, default comercio, fallback por fondos insuficientes, fallback deshabilitado, riel deshabilitado en config, `select_lowest_cost`.

## Referencias

- [Doc/Arquitectura-es.md](../../Doc/Arquitectura-es.md) — decisión D3, diagrama Rail Switcher
- [Doc/Casos-de-Uso-ER-Flujos-es.md](../../Doc/Casos-de-Uso-ER-Flujos-es.md) — UC-02, UC-08
- [domain/README-es.md](../domain/README-es.md) — tipos compartidos
