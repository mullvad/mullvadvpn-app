package net.mullvad.mullvadvpn.usecase.customlists

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.getById
import net.mullvad.mullvadvpn.relaylist.getRelayItemsByCodes
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class CustomListRelayItemsUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository
) {
    fun getRelayItemLocationsForCustomList(
        customListId: CustomListId
    ): Flow<List<RelayItem.Location>> =
        combine(
            customListsRepository.customLists.mapNotNull { it?.getById(customListId) },
            relayListRepository.relayList
        ) { customList, countries ->
            countries.getRelayItemsByCodes(customList.locations)
        }
}
