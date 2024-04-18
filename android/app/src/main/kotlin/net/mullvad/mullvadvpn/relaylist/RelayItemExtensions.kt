package net.mullvad.mullvadvpn.relaylist

import java.lang.IllegalArgumentException
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers

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

private fun RelayItem.hasOwnership(ownershipConstraint: Constraint<Ownership>): Boolean =
    if (ownershipConstraint is Constraint.Only) {
        when (this) {
            is RelayItem.Country -> cities.any { it.hasOwnership(ownershipConstraint) }
            is RelayItem.City -> relays.any { it.hasOwnership(ownershipConstraint) }
            is RelayItem.Relay -> this.ownership == ownershipConstraint.value
            is RelayItem.CustomList -> locations.any { it.hasOwnership(ownershipConstraint) }
        }
    } else {
        true
    }

private fun RelayItem.hasProvider(providersConstraint: Constraint<Providers>): Boolean =
    if (providersConstraint is Constraint.Only) {
        when (this) {
            is RelayItem.Country -> cities.any { it.hasProvider(providersConstraint) }
            is RelayItem.City -> relays.any { it.hasProvider(providersConstraint) }
            is RelayItem.Relay -> providersConstraint.value.providers.contains(providerName)
            is RelayItem.CustomList -> locations.any { it.hasProvider(providersConstraint) }
        }
    } else {
        true
    }

fun RelayItem.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem? =
    when (this) {
        is RelayItem.City -> filterOnOwnershipAndProvider(ownership, providers)
        is RelayItem.Country -> filterOnOwnershipAndProvider(ownership, providers)
        is RelayItem.CustomList -> filterOnOwnershipAndProvider(ownership, providers)
        is RelayItem.Relay -> filterOnOwnershipAndProvider(ownership, providers)
    }

fun RelayItem.CustomList.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.CustomList {
    val newLocations =
        locations.mapNotNull {
            when (it) {
                is RelayItem.City -> it.filterOnOwnershipAndProvider(ownership, providers)
                is RelayItem.Country -> it.filterOnOwnershipAndProvider(ownership, providers)
                is RelayItem.CustomList ->
                    throw IllegalArgumentException("CustomList can't contain CustomList")
                is RelayItem.Relay -> it.filterOnOwnershipAndProvider(ownership, providers)
            }
        }
    return copy(locations = newLocations)
}

fun RelayItem.Country.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Country? {
    val cities = cities.mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
    return if (cities.isNotEmpty()) {
        this.copy(cities = cities)
    } else {
        null
    }
}

private fun RelayItem.City.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.City? {
    val relays = relays.mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
    return if (relays.isNotEmpty()) {
        this.copy(relays = relays)
    } else {
        null
    }
}

private fun RelayItem.Relay.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Relay? {
    return if (hasOwnership(ownership) && hasProvider(providers)) {
        this
    } else {
        null
    }
}
