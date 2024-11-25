package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem

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

fun RelayItem.Location.descendants(): List<RelayItem.Location> {
    val children = children()
    return children + children.flatMap { it.descendants() }
}

fun List<RelayItem.Location>.withDescendants(): List<RelayItem.Location> =
    this + flatMap { it.descendants() }

private fun RelayItem.Location.hasOwnership(ownershipConstraint: Constraint<Ownership>): Boolean =
    if (ownershipConstraint is Constraint.Only) {
        when (this) {
            is RelayItem.Location.Country -> cities.any { it.hasOwnership(ownershipConstraint) }
            is RelayItem.Location.City -> relays.any { it.hasOwnership(ownershipConstraint) }
            is RelayItem.Location.Relay -> this.provider.ownership == ownershipConstraint.value
        }
    } else {
        true
    }

private fun RelayItem.Location.hasProvider(providersConstraint: Constraint<Providers>): Boolean =
    if (providersConstraint is Constraint.Only) {
        when (this) {
            is RelayItem.Location.Country -> cities.any { it.hasProvider(providersConstraint) }
            is RelayItem.Location.City -> relays.any { it.hasProvider(providersConstraint) }
            is RelayItem.Location.Relay ->
                providersConstraint.value.providers.contains(provider.providerId)
        }
    } else {
        true
    }

fun RelayItem.CustomList.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    shouldFilterByDaita: Boolean,
): RelayItem.CustomList {
    val newLocations =
        locations.mapNotNull {
            when (it) {
                is RelayItem.Location.Country ->
                    it.filter(ownership, providers, shouldFilterByDaita)
                is RelayItem.Location.City -> it.filter(ownership, providers, shouldFilterByDaita)
                is RelayItem.Location.Relay -> it.filter(ownership, providers, shouldFilterByDaita)
            }
        }
    return copy(locations = newLocations)
}

fun RelayItem.Location.Country.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    filterDaita: Boolean,
): RelayItem.Location.Country? {
    val cities = cities.mapNotNull { it.filter(ownership, providers, filterDaita) }
    return if (cities.isNotEmpty()) {
        this.copy(cities = cities)
    } else {
        null
    }
}

private fun RelayItem.Location.City.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    shouldFilterByDaita: Boolean,
): RelayItem.Location.City? {
    val relays = relays.mapNotNull { it.filter(ownership, providers, shouldFilterByDaita) }
    return if (relays.isNotEmpty()) {
        this.copy(relays = relays)
    } else {
        null
    }
}

private fun RelayItem.Location.Relay.hasMatchingDaitaSetting(filterDaita: Boolean): Boolean {
    return if (filterDaita) daita else true
}

private fun RelayItem.Location.Relay.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    shouldFilterByDaita: Boolean,
): RelayItem.Location.Relay? {
    return if (
        hasMatchingDaitaSetting(shouldFilterByDaita) &&
            hasOwnership(ownership) &&
            hasProvider(providers)
    ) {
        this
    } else {
        null
    }
}
