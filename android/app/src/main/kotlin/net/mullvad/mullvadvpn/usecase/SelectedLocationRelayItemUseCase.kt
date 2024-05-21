package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
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
    fun selectedRelayItem() = selectedRelayItemWithTitle().map { it?.first }

    fun selectedRelayItemTitle() = selectedRelayItemWithTitle().map { it?.second }

    private fun selectedRelayItemWithTitle() =
        combine(
            customListsRepository.customLists,
            relayListRepository.relayList,
            selectedLocationRepository.selectedLocation
        ) { customLists, relayList, selectedLocation ->
            findSelectedRelayItemWithTitle(selectedLocation, relayList, customLists ?: emptyList())
        }

    private fun findSelectedRelayItemWithTitle(
        locationConstraint: Constraint<RelayItemId>,
        relayCountries: List<RelayItem.Location.Country>,
        customLists: List<CustomList>
    ): Pair<RelayItem, String>? {
        return if (locationConstraint is Constraint.Only) {
            when (val location = locationConstraint.value) {
                is CustomListId -> {
                    customLists
                        .firstOrNull { it.id == location }
                        ?.toRelayItemCustomList(relayCountries)
                        ?.withTitle()
                }
                is GeoLocationId.Hostname -> {
                    relayCountries.findItemForGeoLocationId(location.city)?.let { item ->
                        val city = item as RelayItem.Location.City
                        city.relays.firstOrNull { it.id == location }?.withTitle(city.name)
                    }
                }
                is GeoLocationId -> {
                    relayCountries.findItemForGeoLocationId(location)?.withTitle()
                }
            }
        } else {
            null
        }
    }

    private fun RelayItem.withTitle(cityName: String? = null): Pair<RelayItem, String> =
        when (this) {
            is RelayItem.CustomList -> this to customListName.value
            is RelayItem.Location.City,
            is RelayItem.Location.Country -> this to name
            is RelayItem.Location.Relay -> this to "$cityName ($name)"
        }
}
