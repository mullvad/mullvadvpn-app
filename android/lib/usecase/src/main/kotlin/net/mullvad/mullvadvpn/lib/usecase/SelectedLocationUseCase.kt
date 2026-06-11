package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class SelectedLocationUseCase(
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val multihopActiveUseCase: MultihopActiveUseCase,
) {
    operator fun invoke() =
        combine(
            relayListRepository.selectedLocation,
            wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
            multihopActiveUseCase(),
        ) { selectedLocation, wireguardConstraints, multihopActiveStatus ->
            if (multihopActiveStatus.isActive) {
                RelayItemSelection.Multiple(
                    entryLocation = wireguardConstraints.entryLocation,
                    exitLocation = selectedLocation,
                )
            } else {
                RelayItemSelection.Single(selectedLocation)
            }
        }
}
