package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun CustomList.toRelayItemCustomList(
    relayCountries: List<RelayItem.Location.Country>
): RelayItem.CustomList =
    RelayItem.CustomList(
        customList = this,
        locations = locations.mapNotNull { relayCountries.findByGeoLocationId(it) },
    )

fun List<RelayItem.CustomList>.filterOnSearchTerm(searchTerm: String) =
    if (searchTerm.isNotEmpty()) {
        this.filter { it.name.contains(searchTerm, ignoreCase = true) }
    } else {
        this
    }

fun RelayItem.CustomList.canAddLocation(location: RelayItem) =
    this.locations.none { it.id == location.id } &&
        this.locations.flatMap { it.descendants() }.none { it.id == location.id }

fun List<RelayItem.CustomList>.getById(id: CustomListId) = this.find { it.id == id }

fun List<CustomList>.getById(id: CustomListId) = this.find { it.id == id }
