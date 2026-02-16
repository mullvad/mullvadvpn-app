//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.
#![allow(rustdoc::private_intra_doc_links)]
mod error;
mod relay_selector;

// Re-exports
pub use error::Error;
pub use relay_selector::{
    AdditionalRelayConstraints, AdditionalWireguardConstraints, GetRelay, RETRY_ORDER,
    RelaySelector, SelectedObfuscator, SelectorConfig, detailer, matcher,
    matcher::filter_matching_relay_list, query, relays::WireguardConfig,
};

// TODO: Merge new stuff with old exports.
pub use relay_selector::{Predicate, Reject, Relay, RelayPartitions};
