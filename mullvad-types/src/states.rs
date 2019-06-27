use serde::{Deserialize, Serialize};
use talpid_types::tunnel::TunnelStateTransition;

/// Represents the state the client strives towards.
/// When in `Secured`, the client should keep the computer from leaking and try to
/// establish a VPN tunnel if it is not up.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetState {
    Unsecured,
    Secured,
}

/// Temporary alias used to migrate the usages.
pub type TunnelState = TunnelStateTransition;
