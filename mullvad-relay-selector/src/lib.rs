//! The relay selector picks one or more relays for any Mullvad VPN config.
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
    GetRelay, Relay, RelaySelector, detailer, endpoint_set, query, relays::WireguardConfig,
};
