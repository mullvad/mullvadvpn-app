use std::fmt;

/// Event resulting from a transition to a new tunnel state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
pub enum TunnelStateTransition {
    /// No connection is established and network is unsecured.
    Disconnected,
    /// Network is secured but tunnel is still connecting.
    Connecting,
    /// Tunnel is connected.
    Connected,
    /// Disconnecting tunnel.
    Disconnecting,
    /// Tunnel is disconnected but secured by blocking all connections.
    Blocked(BlockReason),
}

/// Reason for entering the blocked state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "reason", content = "details")]
pub enum BlockReason {
    /// Authentication with remote server failed.
    AuthFailed(String),
    /// Failed to set security policy.
    SetSecurityPolicyError,
    /// Failed to start connection to remote server.
    StartTunnelError,
    /// No relay server matching the current filter parameters.
    NoMatchingRelay,
    /// No account token configured.
    NoAccountToken,
}

impl fmt::Display for BlockReason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            BlockReason::AuthFailed(ref reason) => {
                return write!(
                    formatter,
                    "Authentication with remote server failed: {}",
                    reason
                );
            }
            BlockReason::SetSecurityPolicyError => "Failed to set security policy",
            BlockReason::StartTunnelError => "Failed to start connection to remote server",
            BlockReason::NoMatchingRelay => "No relay server matches the current settings",
            BlockReason::NoAccountToken => "No account token configured",
        };

        write!(formatter, "{}", description)
    }
}
