package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.CustomListId

private fun CustomList.toRelayItemCustomList(
    relayCountries: List<RelayItem.Country>
): RelayItem.CustomList =
    RelayItem.CustomList(
        id = this.id,
        customListName = CustomListName.fromString(name),
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
        this.locations.flatMap { it.descendants() }.none { it.code == location.code }

fun List<RelayItem.CustomList>.getById(id: CustomListId) = this.find { it.id == id }
