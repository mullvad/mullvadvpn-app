package net.mullvad.mullvadvpn.lib.common.util.relaylist

import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListSearchResult
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.search

fun CustomList.toRelayItemCustomList(
    relayCountries: List<RelayItem.Location.Country>
): RelayItem.CustomList =
    RelayItem.CustomList(
        customList = this,
        locations = locations.mapNotNull { relayCountries.findByGeoLocationId(it) },
    )

fun List<RelayItem.CustomList>.filterOnSearchTerm(searchTerm: String): CustomListSearchResult =
    if (searchTerm.isNotEmpty()) {
        val filtered = this.mapNotNull { it.name.search(searchTerm)?.to(it) }
            .sortedByDescending { it.first }

        CustomListSearchResult(
            matchedCustomLists = filtered.map { it.second },
            highlights = filtered.associate { it.second.id to it.first },
        )
    } else {
        CustomListSearchResult(
            matchedCustomLists = this,
            highlights = emptyMap(),
        )
    }

fun RelayItem.CustomList.canAddLocation(location: RelayItem) =
    this.locations.none { it.id == location.id } &&
        this.locations.flatMap { it.descendants() }.none { it.id == location.id }

fun List<RelayItem.CustomList>.getById(id: CustomListId) = this.find { it.id == id }

fun List<CustomList>.getById(id: CustomListId) = this.find { it.id == id }

fun RelayItem.CustomList.onlyContains(relayItem: RelayItem.Location) =
    this.locations.size == 1 && this.locations.first() == relayItem
