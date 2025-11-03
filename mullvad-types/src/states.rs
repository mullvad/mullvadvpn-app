use crate::{features::FeatureIndicators, location::GeoIpLocation};
use either::Either;
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

impl TargetState {
    pub const fn to_strict(
        &self,
    ) -> Either<TargetStateStrict<Unsecured>, TargetStateStrict<Secured>> {
        match self {
            TargetState::Unsecured => Either::Left(TargetStateStrict::<Unsecured>::new()),
            TargetState::Secured => Either::Right(TargetStateStrict::<Secured>::new()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TargetStateStrict<T> {
    _state: std::marker::PhantomData<T>,
}

#[derive(Clone, Copy)]
pub struct Unsecured;

#[derive(Clone, Copy)]
pub struct Secured;

impl TargetStateStrict<Unsecured> {
    const fn new() -> Self {
        Self {
            _state: std::marker::PhantomData,
        }
    }
}

impl TargetStateStrict<Secured> {
    const fn new() -> Self {
        Self {
            _state: std::marker::PhantomData,
        }
    }
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
        #[cfg(not(target_os = "android"))]
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

    /// Returns the geolocation of the tunnel if it exists.
    pub fn get_location(&self) -> Option<&GeoIpLocation> {
        match self {
            TunnelState::Connected { location, .. }
            | TunnelState::Connecting { location, .. }
            | TunnelState::Disconnected { location, .. } => location.as_ref(),
            _ => None,
        }
    }
}
