package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.CustomList

fun CustomList.toRelayItemList(relayCountries: List<RelayCountry>): CustomRelayItemList =
    CustomRelayItemList(
        id = this.id,
        name = this.name,
        locations =
            this.locations.mapNotNull { relayCountries.findItemForGeographicLocationConstraint(it) }
    )

fun List<CustomList>.toRelayItemLists(
    relayCountries: List<RelayCountry>
): List<CustomRelayItemList> = this.map { it.toRelayItemList(relayCountries) }
