package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.UpdateCustomListResult

class CustomListsRepository(private val messageHandler: MessageHandler) {
    suspend fun createCustomList(name: String): CreateCustomListResult {
        val result = messageHandler.trySendRequest(Request.CreateCustomList(name))

        return if (result) {
            messageHandler.events<Event.CreateCustomListResultEvent>().first().result
        } else {
            CreateCustomListResult.Error(CustomListsError.OtherError)
        }
    }

    fun deleteCustomList(id: String) {
        messageHandler.trySendRequest(Request.DeleteCustomList(id))
    }

    suspend fun updateCustomList(customList: CustomList): UpdateCustomListResult {
        val result = messageHandler.trySendRequest(Request.UpdateCustomList(customList))

        return if (result) {
            messageHandler.events<Event.UpdateCustomListResultEvent>().first().result
        } else {
            UpdateCustomListResult.Error(CustomListsError.OtherError)
        }
    }
}
