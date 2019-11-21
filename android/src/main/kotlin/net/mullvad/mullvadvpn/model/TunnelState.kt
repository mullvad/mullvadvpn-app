package net.mullvad.mullvadvpn.model

import net.mullvad.talpid.net.TunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.BlockReason

sealed class TunnelState() {
    class Disconnected() : TunnelState()
    class Connecting(val endpoint: TunnelEndpoint?, val location: GeoIpLocation?) : TunnelState()
    class Connected(val endpoint: TunnelEndpoint, val location: GeoIpLocation?) : TunnelState()
    class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) : TunnelState()
    class Blocked(val reason: BlockReason) : TunnelState()
}
