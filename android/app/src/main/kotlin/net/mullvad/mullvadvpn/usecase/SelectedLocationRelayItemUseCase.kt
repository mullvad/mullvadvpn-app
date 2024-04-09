package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.findItemForGeographicLocationConstraint
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
            findSelectedRelayItem(selectedLocation, relayList.countries, customLists ?: emptyList())
        }

    private fun findSelectedRelayItem(
        locationConstraint: Constraint<LocationConstraint>,
        relayCountries: List<RelayItem.Location.Country>,
        customLists: List<CustomList>
    ): RelayItem? {
        return if (locationConstraint is Constraint.Only) {
            when (val location = locationConstraint.value) {
                is LocationConstraint.CustomList -> {
                    customLists
                        .firstOrNull { it.id == location.listId }
                        ?.toRelayItemCustomList(relayCountries)
                }
                is LocationConstraint.Location -> {
                    relayCountries.findItemForGeographicLocationConstraint(location.location)
                }
            }
        } else {
            null
        }
    }
}
