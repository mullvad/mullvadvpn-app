package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint

fun RelayItem.toLocationConstraint(): LocationConstraint {
    return when (this) {
        is RelayItem.Country -> LocationConstraint.Location(location)
        is RelayItem.City -> LocationConstraint.Location(location)
        is RelayItem.Relay -> LocationConstraint.Location(location)
        is RelayItem.CustomList -> LocationConstraint.CustomList(id)
    }
}

private fun RelayItem.toGeographicLocationConstraint(): GeographicLocationConstraint =
    when (this) {
        is RelayItem.Country -> this.location
        is RelayItem.City -> this.location
        is RelayItem.Relay -> this.location
        is RelayItem.CustomList ->
            throw IllegalArgumentException("CustomList is not a geographic location")
    }

fun List<RelayItem>.toGeographicLocationConstraints(): ArrayList<GeographicLocationConstraint> =
    ArrayList(
        this.map { it.toGeographicLocationConstraint() },
    )
