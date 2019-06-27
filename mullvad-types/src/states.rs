use serde::{Deserialize, Serialize};
use talpid_types::{
    net::TunnelEndpoint,
    tunnel::{ActionAfterDisconnect, BlockReason},
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

/// Represents the state the client tunnel is in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
pub enum TunnelState {
    Disconnected,
    Connecting(TunnelEndpoint),
    Connected(TunnelEndpoint),
    Disconnecting(ActionAfterDisconnect),
    Blocked(BlockReason),
}

impl TunnelState {
    /// Returns true if the tunnel state is the blocked state.
    pub fn is_blocked(&self) -> bool {
        match self {
            TunnelState::Blocked(_) => true,
            _ => false,
        }
    }
}
