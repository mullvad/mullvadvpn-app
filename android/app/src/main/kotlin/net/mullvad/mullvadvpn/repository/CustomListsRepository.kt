package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.CustomList

class CustomListsRepository(private val messageHandler: MessageHandler) {
    suspend fun createCustomList(name: String): String? {
        val result = messageHandler.trySendRequest(Request.CreateCustomList(name))

        return if (result) {
            messageHandler.events<Event.CreateCustomListResultEvent>().first().result
        } else {
            null
        }
    }

    fun deleteCustomList(id: String) {
        messageHandler.trySendRequest(Request.DeleteCustomList(id))
    }

    fun updateCustomList(customList: CustomList) {
        messageHandler.trySendRequest(Request.UpdateCustomList(customList))
    }
}
