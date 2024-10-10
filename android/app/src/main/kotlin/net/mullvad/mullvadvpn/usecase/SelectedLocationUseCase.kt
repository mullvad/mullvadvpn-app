package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.SelectedLocation
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class SelectedLocationUseCase(
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    operator fun invoke(): StateFlow<SelectedLocation> =
        combine(
                relayListRepository.selectedLocation.filterNotNull(),
                wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
            ) { selectedLocation, wireguardConstraints ->
                if (wireguardConstraints.useMultihop) {
                    SelectedLocation.Multiple(
                        entryLocation = wireguardConstraints.entryLocation,
                        exitLocation = selectedLocation,
                    )
                } else {
                    SelectedLocation.Single(selectedLocation)
                }
            }
            .stateIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed(),
                SelectedLocation.Single(Constraint.Any),
            )
}
