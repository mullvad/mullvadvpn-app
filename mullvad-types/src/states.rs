use crate::{features::FeatureIndicators, location::GeoIpLocation};
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::{
    net::{TunnelEndpoint, TunnelType},
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
pub enum TunnelState {
    Disconnected {
        location: Option<GeoIpLocation>,
        /// Whether internet access is blocked due to lockdown mode
        locked_down: bool,
    },
    Connecting {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
        feature_indicators: FeatureIndicators,
    },
    Connected {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
        feature_indicators: FeatureIndicators,
    },
    Disconnecting(ActionAfterDisconnect),
    Error(ErrorState),
}

impl TunnelState {
    /// Returns true if the tunnel state is in the error state.
    pub const fn is_in_error_state(&self) -> bool {
        matches!(self, TunnelState::Error(_))
    }

    /// Returns true if the tunnel state is in the connected state.
    pub const fn is_connected(&self) -> bool {
        matches!(self, TunnelState::Connected { .. })
    }

    /// Returns true if the tunnel state is in the disconnected state.
    pub const fn is_disconnected(&self) -> bool {
        matches!(self, TunnelState::Disconnected { .. })
    }

    /// Returns the tunnel endpoint for an active connection.
    /// This value exists in the connecting and connected states.
    pub const fn endpoint(&self) -> Option<&TunnelEndpoint> {
        match self {
            TunnelState::Connecting { endpoint, .. } | TunnelState::Connected { endpoint, .. } => {
                Some(endpoint)
            }
            _ => None,
        }
    }

    /// Returns the tunnel type for an active connection.
    /// This value exists in the connecting and connected states.
    pub const fn get_tunnel_type(&self) -> Option<TunnelType> {
        match self.endpoint() {
            Some(endpoint) => Some(endpoint.tunnel_type),
            None => None,
        }
    }

    /// Returns the current feature indicators for an active connection.
    /// This value exists in the connecting and connected states.
    pub const fn get_feature_indicators(&self) -> Option<&FeatureIndicators> {
        match self {
            TunnelState::Connecting {
                feature_indicators, ..
            }
            | TunnelState::Connected {
                feature_indicators, ..
            } => Some(feature_indicators),
            _ => None,
        }
    }

    /// Update the set of feature indicators for this [`TunnelState`]. This is only applicable in
    /// the connecting and connected states.
    pub fn set_feature_indicators(self, feature_indicators: FeatureIndicators) -> TunnelState {
        match self {
            TunnelState::Connecting {
                endpoint,
                location,
                feature_indicators: _,
            } => TunnelState::Connecting {
                endpoint,
                location,
                feature_indicators,
            },
            TunnelState::Connected {
                endpoint,
                location,
                feature_indicators: _,
            } => TunnelState::Connected {
                endpoint,
                location,
                feature_indicators,
            },
            _ => self,
        }
    }
}
