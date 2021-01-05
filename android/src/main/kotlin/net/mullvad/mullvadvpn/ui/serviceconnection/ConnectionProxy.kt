package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.util.DispatchingHandler
import net.mullvad.talpid.util.EventNotifier

class ConnectionProxy(val connection: Messenger, eventDispatcher: DispatchingHandler<Event>) {
    val onStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected())

    init {
        eventDispatcher.registerHandler(Event.TunnelStateChange::class) { event ->
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
