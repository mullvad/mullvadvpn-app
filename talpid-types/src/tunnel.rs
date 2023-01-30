use crate::net::TunnelEndpoint;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(target_os = "android")]
use std::net::IpAddr;

/// Event emitted from the states in `talpid_core::tunnel_state_machine` when the tunnel state
/// machine enters a new state.
#[derive(Clone, Debug)]
pub enum TunnelStateTransition {
    /// No connection is established and network is unsecured.
    Disconnected,
    /// Network is secured but tunnel is still connecting.
    Connecting(TunnelEndpoint),
    /// Tunnel is connected.
    Connected(TunnelEndpoint),
    /// Disconnecting tunnel.
    Disconnecting(ActionAfterDisconnect),
    /// Tunnel is disconnected but usually secured by blocking all connections.
    Error(ErrorState),
}

/// Action that will be taken after disconnection is complete.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub enum ActionAfterDisconnect {
    Nothing,
    Block,
    Reconnect,
}

/// Represents the tunnel state machine entering an error state during a [`TunnelStateTransition`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub struct ErrorState {
    /// Reason why the tunnel state machine ended up in the error state
    cause: ErrorStateCause,
    /// Indicates whether the daemon is currently blocking all traffic. This _should_ always
    /// succeed - in the case it does not, the user should be notified that no traffic is being
    /// blocked.
    /// An error value means there was a serious error and the intended security properties are not
    /// being upheld.
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|block_failure| block_failure.is_none()")
    )]
    block_failure: Option<FirewallPolicyError>,
}

impl ErrorState {
    pub fn new(cause: ErrorStateCause, block_failure: Option<FirewallPolicyError>) -> Self {
        Self {
            cause,
            block_failure,
        }
    }

    pub fn is_blocking(&self) -> bool {
        self.block_failure.is_none()
    }

    pub fn cause(&self) -> &ErrorStateCause {
        &self.cause
    }

    pub fn block_failure(&self) -> Option<&FirewallPolicyError> {
        self.block_failure.as_ref()
    }
}

/// Reason for the tunnel state machine entering an [`ErrorState`].
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    SetFirewallPolicyError(FirewallPolicyError),
    /// Failed to set system DNS server.
    SetDnsError,
    /// Android has rejected one or more DNS server addresses.
    #[cfg(target_os = "android")]
    InvalidDnsServers(Vec<IpAddr>),
    /// Failed to start connection to remote server.
    StartTunnelError,
    /// Tunnel parameter generation failure
    TunnelParameterError(ParameterGenerationError),
    /// This device is offline, no tunnels can be established.
    IsOffline,
    /// The Android VPN permission was denied.
    #[cfg(target_os = "android")]
    VpnPermissionDenied,
    /// Error reported by split tunnel module.
    #[cfg(target_os = "windows")]
    SplitTunnelError,
}

impl ErrorStateCause {
    #[cfg(target_os = "macos")]
    pub fn prevents_filtering_resolver(&self) -> bool {
        match self {
            Self::SetDnsError => true,
            _ => false,
        }
    }
}

/// Errors that can occur when generating tunnel parameters.
#[derive(err_derive::Error, Debug, Serialize, Clone, Deserialize)]
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

/// Application that prevents setting the firewall policy.
#[cfg(windows)]
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BlockingApplication {
    pub name: String,
    pub pid: u32,
}

/// Errors that can occur when setting the firewall policy.
#[derive(err_derive::Error, Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "reason", content = "details")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.talpid.tunnel"))]
pub enum FirewallPolicyError {
    /// General firewall failure
    #[error(display = "Failed to set firewall policy")]
    Generic,
    /// An application prevented the firewall policy from being set
    #[cfg(windows)]
    #[error(display = "An application prevented the firewall policy from being set")]
    Locked(Option<BlockingApplication>),
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
            SetFirewallPolicyError(ref err) => {
                return match err {
                    #[cfg(windows)]
                    FirewallPolicyError::Locked(Some(value)) => {
                        write!(f, "{}: {} (pid {})", err, value.name, value.pid)
                    }
                    _ => write!(f, "{err}"),
                };
            }
            SetDnsError => "Failed to set system DNS server",
            #[cfg(target_os = "android")]
            InvalidDnsServers(ref addresses) => {
                return write!(
                    f,
                    "Invalid DNS server addresses used in tunnel configuration: {}",
                    addresses
                        .iter()
                        .map(IpAddr::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            StartTunnelError => "Failed to start connection to remote server",
            TunnelParameterError(ref err) => {
                return write!(f, "Failure to generate tunnel parameters: {err}");
            }
            IsOffline => "This device is offline, no tunnels can be established",
            #[cfg(target_os = "android")]
            VpnPermissionDenied => "The Android VPN permission was denied when creating the tunnel",
            #[cfg(target_os = "windows")]
            SplitTunnelError => "The split tunneling module reported an error",
        };

        write!(f, "{description}")
    }
}
