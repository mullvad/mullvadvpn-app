package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.toGeographicLocationConstraints

class CustomListsRepository(
    private val messageHandler: MessageHandler,
    private val settingsRepository: SettingsRepository
) {
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

    suspend fun updateCustomListLocations(
        id: String,
        locations: List<RelayItem>
    ): UpdateCustomListResult {
        return getCustomListById(id)?.let {
            updateCustomList(it.copy(locations = locations.toGeographicLocationConstraints()))
        } ?: UpdateCustomListResult.Error(CustomListsError.OtherError)
    }

    suspend fun updateCustomListName(id: String, name: String): UpdateCustomListResult {
        return getCustomListById(id)?.let { updateCustomList(it.copy(name = name)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)
    }

    private suspend fun getCustomListById(id: String): CustomList? =
        settingsRepository.settingsUpdates.firstOrNull()?.customLists?.customLists?.find {
            it.id == id
        }
}
