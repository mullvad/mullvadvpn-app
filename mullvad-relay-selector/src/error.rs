//! Definition of relay selector errors

use mullvad_types::{relay_constraints::MissingCustomBridgeSettings, relay_list::Relay};

use crate::WireguardConfig;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open relay cache file")]
    OpenRelayCache(#[source] std::io::Error),

    #[error("Failed to write relay cache file to disk")]
    WriteRelayCache(#[source] std::io::Error),

    #[error("No relays matching current constraints")]
    NoRelay,

    #[error("No bridges matching current constraints")]
    NoBridge,

    #[error("No obfuscators matching current constraints")]
    NoObfuscator,

    #[error("No endpoint could be constructed for relay {0:?}")]
    NoEndpoint(Box<EndpointError>),

    #[error("Failure in serialization of the relay list")]
    Serialize(#[from] serde_json::Error),

    #[error("Downloader already shut down")]
    DownloaderShutDown,

    #[error("Invalid bridge settings")]
    InvalidBridgeSettings(#[from] MissingCustomBridgeSettings),
}

/// Special type which only shows up in [`Error`]. This error variant signals that no valid
/// endpoint could be constructed from the selected relay. See [`detailer`] for more info.
///
/// [`detailer`]: mullvad_relay_selector::relay_selector::detailer.rs
#[derive(Debug)]
pub enum EndpointError {
    /// No valid Wireguard endpoint could be constructed from this [`WireguardConfig`]
    Wireguard(WireguardConfig),
    /// No valid OpenVPN endpoint could be constructed from this [`Relay`]
    OpenVpn(Relay),
}

impl EndpointError {
    /// Helper function for constructing an [`Error::NoEndpoint`] from `relay`.
    /// Takes care of boxing the [`WireguardConfig`] for you!
    pub(crate) fn from_wireguard(relay: WireguardConfig) -> Error {
        Error::NoEndpoint(Box::new(EndpointError::Wireguard(relay)))
    }

    /// Helper function for constructing an [`Error::NoEndpoint`] from `relay`.
    /// Takes care of boxing the [`Relay`] for you!
    pub(crate) fn from_openvpn(relay: Relay) -> Error {
        Error::NoEndpoint(Box::new(EndpointError::OpenVpn(relay)))
    }
}
