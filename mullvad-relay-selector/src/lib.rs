//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.
#![allow(rustdoc::private_intra_doc_links)]
mod constants;
mod error;
#[cfg_attr(target_os = "android", allow(unused))]
mod relay_selector;

// Re-exports
pub use error::Error;
pub use relay_selector::detailer;
pub use relay_selector::query;
pub use relay_selector::relays::WireguardConfig;
pub use relay_selector::{
    AdditionalRelayConstraints, AdditionalWireguardConstraints, GetRelay, RelaySelector,
    RuntimeParameters, SelectedBridge, SelectedObfuscator, SelectorConfig, RETRY_ORDER,
};
