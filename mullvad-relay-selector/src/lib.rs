//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

mod constants;
mod error;
#[cfg_attr(target_os = "android", allow(unused))]
mod relay_selector;

// Re-exports
pub use error::Error;
pub use relay_selector::detailer;
pub use relay_selector::{
    query, GetRelay, RelaySelector, SelectedBridge, SelectedObfuscator, SelectorConfig,
    WireguardConfig, RETRY_ORDER,
};
