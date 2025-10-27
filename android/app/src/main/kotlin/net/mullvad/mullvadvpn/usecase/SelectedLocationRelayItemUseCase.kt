package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase

class SelectedLocationRelayItemUseCase(
    private val customListRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(): Flow<Pair<RelayItem?, RelayItem?>> =
        combine(
            customListRelayItemUseCase(),
            relayListRepository.relayList,
            wireguardConstraintsRepository.wireguardConstraints.map {
                it?.entryLocation
            }.filterNotNull(),
            relayListRepository.selectedLocation,
        ) { customLists, relayList, selectedEntryLocation, selectedExitLocation ->
            selectedEntryLocation.toRelayItem(customLists, relayList) to
                selectedExitLocation.toRelayItem(customLists, relayList)
        }

    private fun Constraint<RelayItemId>.toRelayItem(
        customLists: List<RelayItem.CustomList>,
        relayList: List<RelayItem.Location.Country>,
    ) =
        if (this is Constraint.Only) {
            when (val id = this.value) {
                is CustomListId -> customLists.firstOrNull { it.id == id }
                is GeoLocationId -> relayList.findByGeoLocationId(id)
            }
        } else {
            null
        }
}
