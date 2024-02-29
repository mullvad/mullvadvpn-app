package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.CustomList

private fun CustomList.toRelayItemCustomList(
    relayCountries: List<RelayItem.Country>
): RelayItem.CustomList =
    RelayItem.CustomList(
        id = this.id,
        name = this.name,
        expanded = false,
        locations =
            this.locations.mapNotNull {
                relayCountries.findItemForGeographicLocationConstraint(it)
            },
    )

fun List<CustomList>.toRelayItemLists(
    relayCountries: List<RelayItem.Country>
): List<RelayItem.CustomList> = this.map { it.toRelayItemCustomList(relayCountries) }

fun List<RelayItem.CustomList>.filterOnSearchTerm(searchTerm: String) =
    if (searchTerm.length >= MIN_SEARCH_LENGTH) {
        this.filter { it.name.contains(searchTerm, ignoreCase = true) }
    } else {
        this
    }

fun RelayItem.CustomList.canAddLocation(location: RelayItem) =
    this.locations.none { it.code == location.code } &&
        this.locations.flatMap { it.allChildren() }.none { it.code == location.code }
