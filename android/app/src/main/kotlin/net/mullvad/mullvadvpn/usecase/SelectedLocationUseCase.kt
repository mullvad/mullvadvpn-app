package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class SelectedLocationUseCase(
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke() =
        combine(
            relayListRepository.selectedLocation.filterNotNull(),
            wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
        ) { selectedLocation, wireguardConstraints ->
            if (wireguardConstraints.isMultihopEnabled) {
                RelayItemSelection.Multiple(
                    entryLocation = wireguardConstraints.entryLocation,
                    exitLocation = selectedLocation,
                )
            } else {
                RelayItemSelection.Single(selectedLocation)
            }
        }
}
