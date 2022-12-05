package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.util.trySend

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

    val deviceListUpdates = callbackFlow {
        val handler: (Event.DeviceListUpdate) -> Unit = { event ->
            trySend(event.event)
        }
        dispatcher.registerHandler(Event.DeviceListUpdate::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    val deviceRemovalResult = callbackFlow {
        val handler: (Event.DeviceRemovalEvent) -> Unit = { event ->
            trySend(event)
        }
        dispatcher.registerHandler(Event.DeviceRemovalEvent::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    // Async result: Event.DeviceChanged
    fun refreshDevice() {
        connection.trySend(Request.RefreshDeviceState.message, true)
    }

    fun getDevice() {
        connection.trySend(Request.GetDevice.message, true)
    }

    fun removeDevice(accountToken: String, deviceId: String) {
        connection.trySend(Request.RemoveDevice(accountToken, deviceId).message, true)
    }

    fun refreshDeviceList(accountToken: String) {
        connection.trySend(Request.GetDeviceList(accountToken).message, true)
    }
}
