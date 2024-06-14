package net.mullvad.mullvadvpn.usecase.customlists

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class CustomListsRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
) {

    operator fun invoke() =
        combine(customListsRepository.customLists, relayListRepository.relayList) {
            customLists,
            relayList ->
            customLists?.map { it.toRelayItemCustomList(relayList) } ?: emptyList()
        }
}
