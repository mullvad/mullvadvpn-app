package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.util.trySendRequest

class ServiceConnectionDeviceDataSource(
    private val connection: Messenger,
    private val dispatcher: EventDispatcher
) {
    val deviceStateUpdates = callbackFlow {
        val handler: (Event.DeviceStateEvent) -> Unit = { event ->
            trySend(event.newState)
        }
        dispatcher.registerHandler(Event.DeviceStateEvent::class, handler)
        connection.trySendRequest(Request.GetDevice, false)
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
        connection.trySendRequest(Request.RefreshDeviceState, true)
    }

    fun getDevice() {
        connection.trySendRequest(Request.GetDevice, true)
    }

    fun removeDevice(accountToken: String, deviceId: String) {
        connection.trySendRequest(Request.RemoveDevice(accountToken, deviceId), true)
    }

    fun refreshDeviceList(accountToken: String) {
        connection.trySendRequest(Request.GetDeviceList(accountToken), true)
    }
}
