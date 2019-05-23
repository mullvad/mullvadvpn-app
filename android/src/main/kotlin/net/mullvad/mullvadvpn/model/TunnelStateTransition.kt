package net.mullvad.mullvadvpn.model

sealed class TunnelStateTransition() {
    class Disconnected() : TunnelStateTransition()
    class Connecting() : TunnelStateTransition()
    class Connected() : TunnelStateTransition()
    class Disconnecting() : TunnelStateTransition()
    class Blocked() : TunnelStateTransition()
}
