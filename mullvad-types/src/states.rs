#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DaemonState {
    pub state: SecurityState,
    pub target_state: TargetState,
}

/// Security state of the computer.
/// TODO(linus): There is a difference between lockdown(firewall) and tunnel functionality. The
/// firewall can be set to prevent any leaks but the tunnel is not connected. Then we are secured,
/// but disconnected. The frontend should probably reflect these states in some way. I think it
/// be reasonable to have three states, since unsecured but tunnel is up is probably an invalid
/// state.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityState {
    Unsecured,
    Secured,
}

/// Represents the state the client strives towards.
/// When in `Secured`, the client should keep the computer from leaking and try to
/// establish a VPN tunnel if it is not up.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetState {
    Unsecured,
    Secured,
}
