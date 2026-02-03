package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.common.util.entryBlocked
import net.mullvad.mullvadvpn.lib.common.util.isMultihopEnabled
import net.mullvad.mullvadvpn.lib.common.util.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.lib.common.util.wireguardConstraints
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListsRelayItemUseCase

class HopSelectionUseCase(
    private val customListRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
) {
    operator fun invoke(): Flow<HopSelection> =
        combine(
            customListRelayItemUseCase(),
            relayListRepository.relayList,
            settingsRepository.settingsUpdates.filterNotNull(),
            relayListRepository.selectedLocation,
        ) { customLists, relayList, settings, selectedExitLocation ->
            if (settings.isMultihopEnabled()) {
                val entry =
                    if (settings.entryBlocked()) {
                        Constraint.Any
                    } else {
                        settings
                            .wireguardConstraints()
                            .entryLocation
                            .toRelayItemConstraint(customLists, relayList)
                    }
                HopSelection.Multi(
                    entry,
                    selectedExitLocation.toRelayItemConstraint(customLists, relayList),
                )
            } else {
                HopSelection.Single(
                    selectedExitLocation.toRelayItemConstraint(customLists, relayList)
                )
            }
        }

    private fun Constraint<RelayItemId>.toRelayItemConstraint(
        customLists: List<RelayItem.CustomList>,
        relayList: List<RelayItem.Location.Country>,
    ): Constraint<RelayItem>? =
        if (this is Constraint.Only) {
            when (val id = this.value) {
                is CustomListId -> customLists.firstOrNull { it.id == id }
                is GeoLocationId -> relayList.findByGeoLocationId(id)
            }?.let(Constraint<RelayItem>::Only)
        } else {
            Constraint.Any
        }
}
