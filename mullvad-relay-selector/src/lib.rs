//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.
#![allow(rustdoc::private_intra_doc_links)]
mod error;
mod relay_selector;

// Re-exports
pub use error::Error;
pub use mullvad_types::relay_selector::{
    EntryConstraints, EntrySpecificConstraints, ExitConstraints, MultihopConstraints, Predicate,
    Reason, RelayPartitions,
};
pub use relay_selector::{
    GetRelay, RETRY_ORDER, Relay, RelaySelector, detailer, endpoint_set, query,
    relays::WireguardConfig,
};
