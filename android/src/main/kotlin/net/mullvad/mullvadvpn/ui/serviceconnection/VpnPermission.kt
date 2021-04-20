package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.MessageDispatcher
import net.mullvad.mullvadvpn.ipc.Request

class VpnPermission(private val connection: Messenger, eventDispatcher: MessageDispatcher<Event>) {
    var onRequest: (() -> Unit)? = null

    init {
        eventDispatcher.registerHandler(Event.VpnPermissionRequest::class) { _ ->
            onRequest?.invoke()
        }
    }

    fun grant(isGranted: Boolean) {
        connection.send(Request.VpnPermissionResponse(isGranted).message)
    }
}
