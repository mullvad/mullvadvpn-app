package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnOwnershipAndProvider
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterRepository: RelayListFilterRepository
) {
    fun filteredRelayList() =
        combine(
            relayListRepository.relayList,
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
        ) { relayList, selectedOwnership, selectedProviders ->
            relayList.filterOnOwnershipAndProvider(
                selectedOwnership,
                selectedProviders,
            )
        }

    private fun List<RelayItem.Location.Country>.filterOnOwnershipAndProvider(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ) = mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
}
