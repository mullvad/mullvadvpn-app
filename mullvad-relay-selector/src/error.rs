//! Definition of relay selector errors
#![allow(dead_code)]

use crate::{detailer, relay_selector::relays::WireguardConfig};
use mullvad_types::relay_selector::Reason;
use talpid_types::net::IpVersion;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No relays matching current entry constraints: {0:?}")]
    /// Contains the list of reasons that no relays match. Resolving any of them would yield at least one relay.
    NoRelayEntry(Vec<Reason>),

    #[error("No relays matching current exit constraints: {0:?}")]
    /// Contains the list of reasons that no relays match. Resolving any of them would yield at least one relay.
    NoRelayExit(Vec<Reason>),

    #[error("No relays matching current constraints: {0:?}")]
    /// Contains the list of reasons that no relays match. Resolving any of them would yield at least one relay.
    NoRelay(Vec<Reason>),

    #[error("No bridges matching current constraints")]
    NoBridge,

    #[error("No endpoint could be constructed due to {} for relay {:?}", .internal, .relay)]
    NoEndpoint {
        internal: detailer::Error,
        relay: EndpointErrorDetails,
    },

    #[error("The requested IP version ({family}) does not match ip availability")]
    IpVersionUnavailable { family: IpVersion },
}

/// Special type which only shows up in [`Error`]. This error variant signals that no valid
/// endpoint could be constructed from this [`WireguardConfig`].
///
/// # Note
/// The inner value is boxed to not bloat the size of [`Error`] due to the size of
/// [`WireguardConfig`].
#[derive(Debug)]
pub struct EndpointErrorDetails(Box<WireguardConfig>);

impl EndpointErrorDetails {
    /// Helper function for constructing an [`Error::NoEndpoint`] from `relay`.
    /// Takes care of boxing the [`WireguardConfig`] for you!
    pub(crate) fn from_wireguard(config: WireguardConfig) -> Self {
        EndpointErrorDetails(Box::new(config))
    }
}
