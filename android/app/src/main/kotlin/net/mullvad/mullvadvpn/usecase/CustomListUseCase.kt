package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.toGeographicLocationConstraints
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository

class CustomListUseCase(
    private val customListsRepository: CustomListsRepository,
    private val settingsRepository: SettingsRepository
) {
    suspend fun createCustomList(name: String): String? {
        return customListsRepository.createCustomList(name)
    }

    fun deleteCustomList(id: String) {
        customListsRepository.deleteCustomList(id)
    }

    suspend fun updateCustomListLocations(id: String, locations: List<RelayItem>) {
        getCustomListById(id)?.let {
            customListsRepository.updateCustomList(
                it.copy(locations = locations.toGeographicLocationConstraints())
            )
        }
    }

    suspend fun updateCustomListName(id: String, name: String): Boolean {
        getCustomListById(id)?.let {
            customListsRepository.updateCustomList(it.copy(name = name))
            return true
        } ?: return false
    }

    private suspend fun getCustomListById(id: String): CustomList? =
        settingsRepository.settingsUpdates.firstOrNull()?.customLists?.customLists?.find {
            it.id == id
        }
}
