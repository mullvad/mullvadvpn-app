package net.mullvad.mullvadvpn.usecase.customlists

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.relaylist.toRelayItemLists
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class CustomListsRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
) {

    fun customListsRelayItems() =
        combine(customListsRepository.customLists, relayListRepository.relayList) {
            customLists,
            relayList ->
            customLists?.toRelayItemLists(relayList.countries) ?: emptyList()
        }
}
