use crate::net::TunnelEndpoint;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Event resulting from a transition to a new tunnel state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
pub enum TunnelStateTransition {
    /// No connection is established and network is unsecured.
    Disconnected,
    /// Network is secured but tunnel is still connecting.
    Connecting(TunnelEndpoint),
    /// Tunnel is connected.
    Connected(TunnelEndpoint),
    /// Disconnecting tunnel.
    Disconnecting(ActionAfterDisconnect),
    /// Tunnel is disconnected but secured by blocking all connections.
    Error(ErrorState),
}

/// Action that will be taken after disconnection is complete.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub enum ActionAfterDisconnect {
    Nothing,
    Block,
    Reconnect,
}

/// Error state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub struct ErrorState {
    /// Reason why the tunnel state machine ended up in the error state
    cause: ErrorStateCause,
    /// Indicates whether the daemon is currently blocking all traffic. This _should_ always be
    /// true - in the case it is not, the user should be notified that no traffic is being blocked.
    /// A false value means there was a serious error and the intended security properties are not
    /// being upheld.
    is_blocking: bool,
}

impl ErrorState {
    pub fn new(cause: ErrorStateCause, is_blocking: bool) -> Self {
        Self { cause, is_blocking }
    }

    pub fn is_blocking(&self) -> bool {
        self.is_blocking
    }

    pub fn cause(&self) -> &ErrorStateCause {
        &self.cause
    }
}


/// Reason for entering the blocked state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "reason", content = "details")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub enum ErrorStateCause {
    /// Authentication with remote server failed.
    AuthFailed(Option<String>),
    /// Failed to configure IPv6 because it's disabled in the platform.
    Ipv6Unavailable,
    /// Failed to set firewall policy.
    SetFirewallPolicyError,
    /// Failed to set system DNS server.
    SetDnsError,
    /// Failed to start connection to remote server.
    StartTunnelError,
    /// Tunnel parameter generation failure
    TunnelParameterError(ParameterGenerationError),
    /// This device is offline, no tunnels can be established.
    IsOffline,
    /// A problem with the TAP adapter has been detected.
    TapAdapterProblem,
    /// The Android VPN permission was denied.
    #[cfg(target_os = "android")]
    VpnPermissionDenied,
}

/// Errors that can occur when generating tunnel parameters.
#[derive(err_derive::Error, Debug, Serialize, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub enum ParameterGenerationError {
    /// Failure to select a matching tunnel relay
    #[error(display = "Failure to select a matching tunnel relay")]
    NoMatchingRelay,
    /// Failure to select a matching bridge relay
    #[error(display = "Failure to select a matching bridge relay")]
    NoMatchingBridgeRelay,
    /// Returned when tunnel parameters can't be generated because wireguard key is not available.
    #[error(display = "No wireguard key available")]
    NoWireguardKey,
    /// Failure to resolve the hostname of a custom tunnel configuration
    #[error(display = "Can't resolve hostname for custom tunnel host")]
    CustomTunnelHostResultionError,
}

impl fmt::Display for ErrorStateCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorStateCause::*;
        let description = match *self {
            AuthFailed(ref reason) => {
                return write!(
                    f,
                    "Authentication with remote server failed: {}",
                    match reason {
                        Some(ref reason) => reason.as_str(),
                        None => "No reason provided",
                    }
                );
            }
            Ipv6Unavailable => "Failed to configure IPv6 because it's disabled in the platform",
            SetFirewallPolicyError => "Failed to set firewall policy",
            SetDnsError => "Failed to set system DNS server",
            StartTunnelError => "Failed to start connection to remote server",
            TunnelParameterError(ref err) => {
                return write!(f, "Failure to generate tunnel parameters: {}", err);
            }
            IsOffline => "This device is offline, no tunnels can be established",
            TapAdapterProblem => "A problem with the TAP adapter has been detected",
            #[cfg(target_os = "android")]
            VpnPermissionDenied => "The Android VPN permission was denied when creating the tunnel",
        };

        write!(f, "{}", description)
    }
}
