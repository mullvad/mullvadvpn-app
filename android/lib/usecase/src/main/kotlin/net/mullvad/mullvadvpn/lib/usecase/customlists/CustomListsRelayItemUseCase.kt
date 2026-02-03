package net.mullvad.mullvadvpn.lib.usecase.customlists

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.common.util.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository

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
