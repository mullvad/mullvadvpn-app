package net.mullvad.mullvadvpn.usecase.customlists

import kotlin.collections.mapNotNull
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository

class FilterCustomListsRelayItemUseCase(
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke() =
        combine(
            customListsRelayItemUseCase(),
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            settingsRepository.settingsUpdates,
        ) { customLists, selectedOwnership, selectedProviders, settings ->
            customLists.filterOnOwnershipAndProvider(
                selectedOwnership,
                selectedProviders,
                isDaitaEnabled = settings?.isDaitaEnabled() ?: false,
            )
        }

    private fun List<RelayItem.CustomList>.filterOnOwnershipAndProvider(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        isDaitaEnabled: Boolean,
    ) = mapNotNull { it.filter(ownership, providers, isDaitaEnabled = isDaitaEnabled) }
}
