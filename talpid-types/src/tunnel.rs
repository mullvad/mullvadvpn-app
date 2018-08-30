/// Event resulting from a transition to a new tunnel state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TunnelStateTransition {
    /// No connection is established and network is unsecured.
    Disconnected,
    /// Network is secured but tunnel is still connecting.
    Connecting,
    /// Tunnel is connected.
    Connected,
    /// Disconnecting tunnel.
    Disconnecting,
}
