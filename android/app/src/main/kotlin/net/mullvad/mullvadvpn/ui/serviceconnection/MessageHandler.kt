package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

interface MessageHandler {
    val events: Flow<Event>

    fun trySendRequest(request: Request): Boolean
}
