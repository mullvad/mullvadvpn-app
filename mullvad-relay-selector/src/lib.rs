//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.
#![allow(rustdoc::private_intra_doc_links)]
mod constants;
mod error;
#[cfg_attr(target_os = "android", allow(unused))]
mod relay_selector;

// Re-exports
pub use error::Error;
pub use relay_selector::{
    AdditionalRelayConstraints, AdditionalWireguardConstraints, GetRelay, RelaySelector,
    SelectedObfuscator, SelectorConfig, WIREGUARD_RETRY_ORDER, detailer, matcher,
    matcher::filter_matching_relay_list, query, relays::WireguardConfig,
};
