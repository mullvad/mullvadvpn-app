package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.shouldFilterByDaita

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(relayListType: RelayListType) =
        combine(
            relayListRepository.relayList,
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            settingsRepository.settingsUpdates,
            wireguardConstraintsRepository.wireguardConstraints,
        ) { relayList, selectedOwnership, selectedProviders, settings, wireguardConstraints ->
            relayList.filter(
                ownership = selectedOwnership,
                providers = selectedProviders,
                shouldFilterByDaita =
                    shouldFilterByDaita(
                        isDaitaEnabled = settings?.isDaitaEnabled() == true,
                        isMultihopEnabled = wireguardConstraints?.isMultihopEnabled == true,
                        relayListType = relayListType,
                    ),
            )
        }

    private fun List<RelayItem.Location.Country>.filter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        shouldFilterByDaita: Boolean,
    ) = mapNotNull { it.filter(ownership, providers, shouldFilterByDaita) }
}
