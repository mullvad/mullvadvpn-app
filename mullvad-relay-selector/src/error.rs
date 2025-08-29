//! Definition of relay selector errors
#![allow(dead_code)]

use crate::{detailer, relay_selector::relays::WireguardConfig};
use mullvad_types::{relay_constraints::MissingCustomBridgeSettings, relay_list::Relay};
use talpid_types::net::IpVersion;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open relay cache file")]
    OpenRelayCache(#[source] std::io::Error),

    #[error("Failed to write relay cache file to disk")]
    WriteRelayCache(#[source] std::io::Error),

    #[error("The combination of relay constraints is invalid")]
    InvalidConstraints,

    #[error("No relays matching current constraints")]
    NoRelay,

    #[error("No bridges matching current constraints")]
    NoBridge,

    #[error("No obfuscators matching current constraints")]
    NoObfuscator(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("No endpoint could be constructed due to {} for relay {:?}", .internal, .relay)]
    NoEndpoint {
        internal: detailer::Error,
        relay: EndpointErrorDetails,
    },

    #[error("Failure in serialization of the relay list")]
    Serialize(#[from] serde_json::Error),

    #[error("Invalid bridge settings")]
    InvalidBridgeSettings(#[from] MissingCustomBridgeSettings),

    #[error("The requested IP version ({family}) does not match ip availability")]
    IpVersionUnavailable { family: IpVersion },
}

/// Special type which only shows up in [`Error`]. This error variant signals that no valid
/// endpoint could be constructed from the selected relay.
#[derive(Debug)]
pub enum EndpointErrorDetails {
    /// No valid Wireguard endpoint could be constructed from this [`WireguardConfig`].
    ///
    /// # Note
    /// The inner value is boxed to not bloat the size of [`Error`] due to the size of
    /// [`WireguardConfig`].
    Wireguard(Box<WireguardConfig>),
    /// No valid OpenVPN endpoint could be constructed from this [`Relay`]
    ///
    /// # Note
    /// The inner value is boxed to not bloat the size of [`Error`] due to the size of [`Relay`].
    OpenVpn(Box<Relay>),
}

impl EndpointErrorDetails {
    /// Helper function for constructing an [`Error::NoEndpoint`] from `relay`.
    /// Takes care of boxing the [`WireguardConfig`] for you!
    pub(crate) fn from_wireguard(relay: WireguardConfig) -> Self {
        EndpointErrorDetails::Wireguard(Box::new(relay))
    }

    /// Helper function for constructing an [`Error::NoEndpoint`] from `relay`.
    /// Takes care of boxing the [`Relay`] for you!
    pub(crate) fn from_openvpn(relay: Relay) -> Self {
        EndpointErrorDetails::OpenVpn(Box::new(relay))
    }
}
