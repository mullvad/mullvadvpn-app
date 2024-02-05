package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.CustomList

fun CustomList.toRelayItemCustomList(
    relayCountries: List<RelayItem.Country>
): RelayItem.CustomList =
    RelayItem.CustomList(
        code = this.id,
        id = this.id,
        name = this.name,
        expanded = false,
        locations =
            this.locations.mapNotNull { relayCountries.findItemForGeographicLocationConstraint(it) }
    )

fun List<CustomList>.toRelayItemLists(
    relayCountries: List<RelayItem.Country>
): List<RelayItem.CustomList> = this.map { it.toRelayItemCustomList(relayCountries) }
