//! Motor de decisión de riel — preferencia, disponibilidad, costo y fallback (D3).

mod config;
mod switcher;

pub use config::{RailAvailability, RailConfig, RailPreference, RailSelectionInput};
pub use switcher::RailSwitcher;
