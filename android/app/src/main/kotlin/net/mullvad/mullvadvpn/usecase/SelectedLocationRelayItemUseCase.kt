package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.findItemForGeoLocationId
import net.mullvad.mullvadvpn.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SelectedLocationRepository

class SelectedLocationRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
    private val selectedLocationRepository: SelectedLocationRepository
) {
    fun selectedRelayItem() =
        combine(
            customListsRepository.customLists,
            relayListRepository.relayList,
            selectedLocationRepository.selectedLocation
        ) { customLists, relayList, selectedLocation ->
            findSelectedRelayItem(selectedLocation, relayList, customLists ?: emptyList())
        }

    private fun findSelectedRelayItem(
        locationConstraint: Constraint<RelayItemId>,
        relayCountries: List<RelayItem.Location.Country>,
        customLists: List<CustomList>
    ): RelayItem? {
        return if (locationConstraint is Constraint.Only) {
            when (val location = locationConstraint.value) {
                is CustomListId -> {
                    customLists
                        .firstOrNull { it.id == location }
                        ?.toRelayItemCustomList(relayCountries)
                }
                is GeoLocationId -> {
                    relayCountries.findItemForGeoLocationId(location)
                }
            }
        } else {
            null
        }
    }
}
