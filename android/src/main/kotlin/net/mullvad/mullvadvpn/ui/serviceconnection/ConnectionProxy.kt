package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.Event.TunnelStateChange
import net.mullvad.mullvadvpn.service.Event.Type
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.talpid.util.EventNotifier

class ConnectionProxy(val connection: Messenger, eventDispatcher: EventDispatcher) {
    val onStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected())

    init {
        eventDispatcher.registerHandler(Type.TunnelStateChange) { event: TunnelStateChange ->
            onStateChange.notify(event.tunnelState)
        }
    }

    fun connect() {
        connection.send(Request.Connect().message)
    }

    fun disconnect() {
        connection.send(Request.Disconnect().message)
    }

    fun reconnect() {
        connection.send(Request.Reconnect().message)
    }

    fun onDestroy() {
        onStateChange.unsubscribeAll()
    }
}
