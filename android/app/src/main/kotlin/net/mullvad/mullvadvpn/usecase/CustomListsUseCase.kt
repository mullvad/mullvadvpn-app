package net.mullvad.mullvadvpn.usecase

import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class CustomListsUseCase(private val customListsRepository: CustomListsRepository) {
    suspend fun createCustomList(name: String): String? {
        return customListsRepository.createCustomList(name)
    }

    fun deleteCustomList(id: String) {
        customListsRepository.deleteCustomList(id)
    }

    fun updateCustomList(customList: CustomList) {
        customListsRepository.updateCustomList(customList)
    }
}
