//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

mod constants;
mod error;
mod relay_selector;

// Re-exports
pub use crate::relay_selector::query;
pub use error::Error;
pub use relay_selector::{
    GetRelay, RelaySelector, SelectedBridge, SelectedObfuscator, SelectorConfig, WireguardConfig,
    RETRY_ORDER,
};
