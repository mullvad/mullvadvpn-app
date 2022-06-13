package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request

class ServiceConnectionDeviceDataSource(
    private val connection: Messenger,
    private val dispatcher: EventDispatcher
) {
    val deviceStateUpdates = callbackFlow {
        val handler: (Event.DeviceStateEvent) -> Unit = { event ->
            trySend(event.newState)
        }
        dispatcher.registerHandler(Event.DeviceStateEvent::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    // Async result: Event.DeviceChanged
    fun refreshDevice() {
        connection.send(Request.RefreshDeviceState.message)
    }
}
