package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy

class TunnelStateUpdater(context: Context, private val connectionProxy: ConnectionProxy) {
    private val persistence = Persistence(context)

    init {
        connectionProxy.onStateChange.subscribe(this) { newState ->
            persistence.state = newState
        }
    }
}
