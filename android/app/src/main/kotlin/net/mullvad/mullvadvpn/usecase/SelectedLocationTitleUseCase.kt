package net.mullvad.mullvadvpn.usecase

import arrow.core.raise.nullable
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class SelectedLocationTitleUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
) {
    operator fun invoke() =
        combine(
            customListsRepository.customLists,
            relayListRepository.relayList,
            relayListRepository.selectedLocation,
        ) { customLists, relayList, selectedLocation ->
            if (selectedLocation is Constraint.Only) {
                createRelayItemTitle(selectedLocation.value, relayList, customLists ?: emptyList())
            } else {
                null
            }
        }

    private fun createRelayItemTitle(
        relayItemId: RelayItemId,
        relayCountries: List<RelayItem.Location.Country>,
        customLists: List<CustomList>,
    ): String? =
        when (relayItemId) {
            is CustomListId -> customLists.firstOrNull { it.id == relayItemId }?.name?.value
            is GeoLocationId.Hostname -> createRelayTitle(relayCountries, relayItemId)
            is GeoLocationId.City -> relayCountries.findByGeoLocationId(relayItemId)?.name
            is GeoLocationId.Country -> relayCountries.firstOrNull { it.id == relayItemId }?.name
        }

    private fun createRelayTitle(
        relayCountries: List<RelayItem.Location.Country>,
        relayItemId: GeoLocationId.Hostname,
    ): String? = nullable {
        val city = relayCountries.findByGeoLocationId(relayItemId.city).bind()
        val relay = city.relays.firstOrNull { it.id == relayItemId }.bind()

        relay.formatTitle(city)
    }

    private fun RelayItem.Location.Relay.formatTitle(city: RelayItem.Location.City) =
        "${city.name} (${name})"
}
