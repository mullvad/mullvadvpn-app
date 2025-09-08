package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.entryLocation
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled
import net.mullvad.mullvadvpn.util.isMultihopEnabled

class SelectedLocationUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
) {
    operator fun invoke() =
        combine(
            relayListRepository.selectedLocation.filterNotNull(),
            settingsRepository.settingsUpdates.filterNotNull(),
        ) { selectedLocation, settings ->
            if (settings.isMultihopEnabled()) {
                RelayItemSelection.Multiple(
                    entryLocation =
                        if (settings.isDaitaEnabled() && !settings.isDaitaDirectOnly()) {
                            // If Daita is enabled without direct only the app acts as though any
                            // entry location with DAITA is allowed
                            Constraint.Any
                        } else {
                            settings.entryLocation()
                        },
                    exitLocation = selectedLocation,
                )
            } else {
                RelayItemSelection.Single(selectedLocation)
            }
        }
}
