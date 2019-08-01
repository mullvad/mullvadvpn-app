package net.mullvad.mullvadvpn.model

sealed class TunnelState() {
    class Disconnected() : TunnelState()
    class Connecting(val endpoint: TunnelEndpoint?, val location: GeoIpLocation?) : TunnelState()
    class Connected(val endpoint: TunnelEndpoint, val location: GeoIpLocation?) : TunnelState()
    class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) : TunnelState()
    class Blocked(val reason: BlockReason) : TunnelState()
}
