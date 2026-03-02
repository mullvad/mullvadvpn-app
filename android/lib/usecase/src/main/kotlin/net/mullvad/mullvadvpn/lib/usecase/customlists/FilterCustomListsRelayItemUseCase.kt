package net.mullvad.mullvadvpn.lib.usecase.customlists

import kotlin.collections.map
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.usecase.FilteredRelayListUseCase

class FilterCustomListsRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
) {

    operator fun invoke(relayListType: RelayListType) =
        combine(customListsRepository.customLists, filteredRelayListUseCase(relayListType)) {
            customLists,
            filteredRelayList ->
            customLists?.map { it.toRelayItemCustomList(filteredRelayList) } ?: emptyList()
        }
}
