package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class SelectedLocationUseCase(
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val multihopInEffectUseCase: MultihopInEffectUseCase,
) {
    operator fun invoke() =
        combine(
            relayListRepository.selectedLocation,
            wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
            multihopInEffectUseCase(),
        ) { selectedLocation, wireguardConstraints, multihopInEffect ->
            if (multihopInEffect.isInEffect) {
                RelayItemSelection.Multiple(
                    entryLocation = wireguardConstraints.entryLocation,
                    exitLocation = selectedLocation,
                )
            } else {
                RelayItemSelection.Single(selectedLocation)
            }
        }
}
