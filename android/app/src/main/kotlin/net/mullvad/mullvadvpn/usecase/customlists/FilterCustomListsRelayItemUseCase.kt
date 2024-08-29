package net.mullvad.mullvadvpn.usecase.customlists

import kotlin.collections.mapNotNull
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnOwnershipAndProvider
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository

class FilterCustomListsRelayItemUseCase(
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
) {

    operator fun invoke() =
        combine(
            customListsRelayItemUseCase(),
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
        ) { customLists, selectedOwnership, selectedProviders ->
            customLists.filterOnOwnershipAndProvider(selectedOwnership, selectedProviders)
        }

    private fun List<RelayItem.CustomList>.filterOnOwnershipAndProvider(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
    ) = mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
}
