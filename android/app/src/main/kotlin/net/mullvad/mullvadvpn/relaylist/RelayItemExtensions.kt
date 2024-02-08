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
