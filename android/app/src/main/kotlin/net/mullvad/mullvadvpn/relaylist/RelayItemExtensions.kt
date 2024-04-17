package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.RelayItem

fun RelayItem.children(): List<RelayItem> {
    return when (this) {
        is RelayItem.Location.Country -> cities
        is RelayItem.Location.City -> relays
        is RelayItem.CustomList -> locations
        else -> emptyList()
    }
}

fun RelayItem.Location.children(): List<RelayItem.Location> {
    return when (this) {
        is RelayItem.Location.Country -> cities
        is RelayItem.Location.City -> relays
        else -> emptyList()
    }
}

fun RelayItem.descendants(): List<RelayItem> {
    val children = children()
    return children + children.flatMap { it.descendants() }
}

fun RelayItem.Location.descendants(): List<RelayItem.Location> {
    val children = children()
    return children + children.flatMap { it.descendants() }
}

fun RelayItem.Location.Country.withDescendants(): List<RelayItem.Location> {
    return listOf(this) + descendants()
}

fun List<RelayItem.Location.Country>.withDescendants(): List<RelayItem.Location> =
    this + flatMap { it.descendants() }
