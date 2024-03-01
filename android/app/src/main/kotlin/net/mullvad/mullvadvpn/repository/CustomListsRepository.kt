package net.mullvad.mullvadvpn.repository

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
import net.mullvad.mullvadvpn.relaylist.getGeographicLocationConstraintByCode
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.util.firstOrNullWithTimeout

class CustomListsRepository(
    private val messageHandler: MessageHandler,
    private val settingsRepository: SettingsRepository,
    private val relayListListener: RelayListListener
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

    private suspend fun updateCustomList(customList: CustomList): UpdateCustomListResult {
        val result = messageHandler.trySendRequest(Request.UpdateCustomList(customList))

        return if (result) {
            messageHandler.events<Event.UpdateCustomListResultEvent>().first().result
        } else {
            UpdateCustomListResult.Error(CustomListsError.OtherError)
        }
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

    private suspend fun updateCustomListLocations(
        id: String,
        locations: ArrayList<GeographicLocationConstraint>
    ): UpdateCustomListResult {
        return awaitCustomListById(id)?.let { updateCustomList(it.copy(locations = locations)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)
    }

    private suspend fun awaitCustomListById(id: String): CustomList? =
        settingsRepository.settingsUpdates
            .mapNotNull { settings -> settings?.customLists?.customLists?.find { it.id == id } }
            .firstOrNullWithTimeout(GET_CUSTOM_LIST_TIMEOUT_MS)

    fun getCustomListById(id: String): CustomList? =
        settingsRepository.settingsUpdates.value?.customLists?.customLists?.find { it.id == id }

    private fun getGeographicLocationConstraintByCode(code: String): GeographicLocationConstraint? =
        relayListListener.relayListEvents.value.getGeographicLocationConstraintByCode(code)

    companion object {
        private const val GET_CUSTOM_LIST_TIMEOUT_MS = 5000L
    }
}
