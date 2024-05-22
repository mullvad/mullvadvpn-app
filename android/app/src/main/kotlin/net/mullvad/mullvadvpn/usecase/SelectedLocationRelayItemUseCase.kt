package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.findItemForGeoLocationId
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectedLocationRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
) {
    fun selectedRelayItemTitle() = selectedRelayItemWithTitle()

    private fun selectedRelayItemWithTitle() =
        combine(
            customListsRepository.customLists,
            relayListRepository.relayList,
            relayListRepository.selectedLocation
        ) { customLists, relayList, selectedLocation ->
            findSelectedRelayItemWithTitle(selectedLocation, relayList, customLists ?: emptyList())
        }

    private fun findSelectedRelayItemWithTitle(
        locationConstraint: Constraint<RelayItemId>,
        relayCountries: List<RelayItem.Location.Country>,
        customLists: List<CustomList>
    ): String? {
        if (locationConstraint !is Constraint.Only) return null

        return when (val relayItemId = locationConstraint.value) {
            is CustomListId -> customLists.firstOrNull { it.id == relayItemId }?.name?.value
            is GeoLocationId.Hostname -> {
                val city =
                    relayCountries
                        .withDescendants()
                        .filterIsInstance<RelayItem.Location.City>()
                        .firstOrNull { it.id == relayItemId.city } ?: return null

                val relay = city.relays.firstOrNull { it.id == relayItemId } ?: return null

                "${city.name} (${relay.name})"
            }
            is GeoLocationId.City -> relayCountries.findItemForGeoLocationId(relayItemId)?.name
            is GeoLocationId.Country -> relayCountries.findItemForGeoLocationId(relayItemId)?.name
        }
    }
}
