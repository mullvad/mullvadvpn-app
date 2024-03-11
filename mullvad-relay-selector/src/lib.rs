//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

mod constants;
mod error;
#[cfg(not(target_os = "android"))]
mod relay_selector;
#[cfg(target_os = "android")]
#[allow(unused)]
mod relay_selector;

// Re-exports
pub use error::Error;
pub use relay_selector::{
    query, GetRelay, RelaySelector, SelectedBridge, SelectedObfuscator, SelectorConfig,
    WireguardConfig, RETRY_ORDER,
};
