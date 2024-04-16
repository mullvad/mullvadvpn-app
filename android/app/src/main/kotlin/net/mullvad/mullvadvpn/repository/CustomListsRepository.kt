package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.getGeographicLocationConstraintByCode
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.util.firstOrNullWithTimeout

class CustomListsRepository(
    private val settingsRepository: SettingsRepository,
    private val relayListListener: RelayListListener
) {
    suspend fun createCustomList(name: CustomListName): CreateCustomListResult {
//        val result = messageHandler.trySendRequest(Request.CreateCustomList(name.value))
//
//        return if (result) {
//            messageHandler.events<Event.CreateCustomListResultEvent>().first().result
//        } else {
//            CreateCustomListResult.Error(CustomListsError.OtherError)
//        }
        TODO()
    }

    fun deleteCustomList(id: String): Unit = TODO()// messageHandler.trySendRequest(Request.DeleteCustomList(id))

    private suspend fun updateCustomList(customList: CustomList): UpdateCustomListResult {
//        val result = messageHandler.trySendRequest(Request.UpdateCustomList(customList))
//
//        return if (result) {
//            messageHandler.events<Event.UpdateCustomListResultEvent>().first().result
//        } else {
//            UpdateCustomListResult.Error(CustomListsError.OtherError)
//        }
        TODO()
    }

    suspend fun updateCustomListLocationsFromCodes(
        id: String,
        locationCodes: List<String>
    ): UpdateCustomListResult =
        updateCustomListLocations(
            id = id,
            locations =
                ArrayList(locationCodes.mapNotNull { getGeographicLocationConstraintByCode(it) })
        )

    suspend fun updateCustomListName(id: String, name: CustomListName): UpdateCustomListResult =
        getCustomListById(id)?.let { updateCustomList(it.copy(name = name.value)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)

    private suspend fun updateCustomListLocations(
        id: String,
        locations: ArrayList<GeographicLocationConstraint>
    ): UpdateCustomListResult =
        awaitCustomListById(id)?.let { updateCustomList(it.copy(locations = locations)) }
            ?: UpdateCustomListResult.Error(CustomListsError.OtherError)

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
