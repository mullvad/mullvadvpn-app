use crate::location::GeoIpLocation;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::{
    net::TunnelEndpoint,
    tunnel::{ActionAfterDisconnect, ErrorState},
};

/// Represents the state the client strives towards.
/// When in `Secured`, the client should keep the computer from leaking and try to
/// establish a VPN tunnel if it is not up.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetState {
    Unsecured,
    Secured,
}

impl fmt::Display for TargetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetState::Unsecured => "Unsecured".fmt(f),
            TargetState::Secured => "Secured".fmt(f),
        }
    }
}

/// Represents the state the client tunnel is in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum TunnelState {
    Disconnected,
    Connecting {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
    },
    Connected {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
    },
    Disconnecting(ActionAfterDisconnect),
    Error(ErrorState),
}

impl TunnelState {
    /// Returns true if the tunnel state is in the error state.
    pub fn is_in_error_state(&self) -> bool {
        matches!(self, TunnelState::Error(_))
    }

    /// Returns true if the tunnel state is in the connected state.
    pub fn is_connected(&self) -> bool {
        matches!(self, TunnelState::Connected { .. })
    }

    /// Returns true if the tunnel state is in the disconnected state.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, TunnelState::Disconnected)
    }
}
