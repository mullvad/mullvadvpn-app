package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.DeadObjectException
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
        try {
            connection.send(Request.RefreshDeviceState.message)
        } catch (ex: DeadObjectException) {
            // inform main controller to recreate connection thread
        }
    }

    fun getDevice() {
        try {
            connection.send(Request.GetDevice.message)
        } catch (ex: DeadObjectException) {
            // inform main controller to recreate connection thread
        }
    }

    fun removeDevice(accountToken: String, deviceId: String) {
        try {
            connection.send(Request.RemoveDevice(accountToken, deviceId).message)
        } catch (ex: DeadObjectException) {
            // inform main controller to recreate connection thread
        }
    }

    fun refreshDeviceList(accountToken: String) {
        try {
            connection.send(Request.GetDeviceList(accountToken).message)
        } catch (ex: DeadObjectException) {
            // inform main controller to recreate connection thread
        }
    }
}
