package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val settingsRepository: SettingsRepository,
) {
    operator fun invoke() =
        combine(
            relayListRepository.relayList,
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            settingsRepository.settingsUpdates,
        ) { relayList, selectedOwnership, selectedProviders, settings ->
            relayList.filter(
                selectedOwnership,
                selectedProviders,
                isDaitaEnabled = settings?.isDaitaEnabled() ?: false,
            )
        }

    private fun List<RelayItem.Location.Country>.filter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        isDaitaEnabled: Boolean,
    ) = mapNotNull { it.filter(ownership, providers, isDaitaEnabled) }
}
