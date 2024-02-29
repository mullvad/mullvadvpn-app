package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.getGeographicLocationConstraintByCode
import net.mullvad.mullvadvpn.relaylist.toGeographicLocationConstraints
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener

class CustomListsRepository(
    private val messageHandler: MessageHandler,
    private val settingsRepository: SettingsRepository,
    private val relayListListener: RelayListListener
) {
    private val latestDeletedCustomList: Channel<CustomList> =
        Channel(capacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)

    suspend fun createCustomList(name: String): CreateCustomListResult {
        val result = messageHandler.trySendRequest(Request.CreateCustomList(name))

        return if (result) {
            messageHandler.events<Event.CreateCustomListResultEvent>().first().result
        } else {
            CreateCustomListResult.Error(CustomListsError.OtherError)
        }
    }

    fun deleteCustomList(id: String) {
        val customList = getCustomListById(id)
        if (messageHandler.trySendRequest(Request.DeleteCustomList(id))) {
            customList?.let { latestDeletedCustomList.trySend(it) }
        }
    }

    private suspend fun updateCustomList(customList: CustomList): UpdateCustomListResult {
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
        return updateCustomListLocations(
            id = id,
            locations = locations.toGeographicLocationConstraints()
        )
    }

    suspend fun updateCustomListLocationsFromCodes(
        id: String,
        locationCodes: List<String>
    ): UpdateCustomListResult {
        return updateCustomListLocations(
            id = id,
            locations =
                ArrayList(locationCodes.mapNotNull { getGeographicLocationConstraintByCode(it) })
        )
    }

    suspend fun updateCustomListName(id: String, name: String): UpdateCustomListResult {
        return getCustomListById(id)?.let { updateCustomList(it.copy(name = name)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)
    }

    suspend fun undoDeleteLatest() {
        latestDeletedCustomList.tryReceive().getOrNull()?.let { deletedCustomList ->
            val result = createCustomList(deletedCustomList.name)
            if (result is CreateCustomListResult.Ok) {
                updateCustomList(deletedCustomList.copy(id = result.id))
            }
        }
    }

    private suspend fun updateCustomListLocations(
        id: String,
        locations: ArrayList<GeographicLocationConstraint>
    ): UpdateCustomListResult {
        return getCustomListById(id)?.let { updateCustomList(it.copy(locations = locations)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)
    }

    private fun getCustomListById(id: String): CustomList? =
        settingsRepository.settingsUpdates.value?.customLists?.customLists?.find { it.id == id }

    private fun getGeographicLocationConstraintByCode(code: String): GeographicLocationConstraint? =
        relayListListener.relayListEvents.value.getGeographicLocationConstraintByCode(code)
}
