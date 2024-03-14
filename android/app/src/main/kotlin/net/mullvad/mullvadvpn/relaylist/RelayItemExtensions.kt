package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

fun RelayItem.toLocationConstraint(): LocationConstraint {
    return when (this) {
        is RelayItem.Country -> LocationConstraint.Location(location)
        is RelayItem.City -> LocationConstraint.Location(location)
        is RelayItem.Relay -> LocationConstraint.Location(location)
        is RelayItem.CustomList -> LocationConstraint.CustomList(id)
    }
}

fun RelayItem.children(): List<RelayItem> {
    return when (this) {
        is RelayItem.Country -> cities
        is RelayItem.City -> relays
        is RelayItem.CustomList -> locations
        else -> emptyList()
    }
}

fun RelayItem.descendants(): List<RelayItem> {
    val children = children()
    return children + children.flatMap { it.descendants() }
}
