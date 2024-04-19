package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
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

fun RelayItem.Location.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Location? =
    when (this) {
        is RelayItem.Location.Country -> this.filterOnOwnershipAndProvider(ownership, providers)
        is RelayItem.Location.City -> this.filterOnOwnershipAndProvider(ownership, providers)
        is RelayItem.Location.Relay -> this.filterOnOwnershipAndProvider(ownership, providers)
    }

fun RelayItem.CustomList.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.CustomList {
    val newLocations =
        locations.mapNotNull {
            when (it) {
                is RelayItem.Location.City -> it.filterOnOwnershipAndProvider(ownership, providers)
                is RelayItem.Location.Country ->
                    it.filterOnOwnershipAndProvider(
                        ownership,
                        providers,
                    )
                is RelayItem.Location.Relay -> it.filterOnOwnershipAndProvider(ownership, providers)
            }
        }
    return copy(locations = newLocations)
}

fun RelayItem.Location.Country.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Location.Country? {
    val cities = cities.mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
    return if (cities.isNotEmpty()) {
        this.copy(cities = cities)
    } else {
        null
    }
}

private fun RelayItem.Location.City.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Location.City? {
    val relays = relays.mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
    return if (relays.isNotEmpty()) {
        this.copy(relays = relays)
    } else {
        null
    }
}

private fun RelayItem.Location.Relay.filterOnOwnershipAndProvider(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): RelayItem.Location.Relay? {
    return if (hasOwnership(ownership) && hasProvider(providers)) {
        this
    } else {
        null
    }
}

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
