package net.mullvad.mullvadvpn.model

sealed class TunnelState() {
    class Disconnected() : TunnelState()
    class Connecting() : TunnelState()
    class Connected() : TunnelState()
    class Disconnecting() : TunnelState()
    class Blocked() : TunnelState()
}
