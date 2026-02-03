package net.mullvad.mullvadvpn.lib.common.util.relaylist

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.Quic
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
            is RelayItem.Location.Relay -> ownershipConstraint.value == ownership
        }
    } else {
        true
    }

private fun RelayItem.Location.hasProvider(providersConstraint: Constraint<Providers>): Boolean =
    if (providersConstraint is Constraint.Only) {
        when (this) {
            is RelayItem.Location.Country -> cities.any { it.hasProvider(providersConstraint) }
            is RelayItem.Location.City -> relays.any { it.hasProvider(providersConstraint) }
            is RelayItem.Location.Relay -> provider in providersConstraint.value
        }
    } else {
        true
    }

fun RelayItem.CustomList.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    daita: Boolean,
    quic: Boolean,
    lwo: Boolean,
    ipVersion: Constraint<IpVersion>,
): RelayItem.CustomList {
    val newLocations =
        locations.mapNotNull {
            when (it) {
                is RelayItem.Location.Country ->
                    it.filter(ownership, providers, daita, quic, lwo, ipVersion)
                is RelayItem.Location.City ->
                    it.filter(ownership, providers, daita, quic, lwo, ipVersion)
                is RelayItem.Location.Relay ->
                    it.filter(ownership, providers, daita, quic, lwo, ipVersion)
            }
        }
    return copy(locations = newLocations)
}

fun RelayItem.Location.Country.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    daita: Boolean,
    quic: Boolean,
    lwo: Boolean,
    ipVersion: Constraint<IpVersion>,
): RelayItem.Location.Country? {
    val cities = cities.mapNotNull { it.filter(ownership, providers, daita, quic, lwo, ipVersion) }
    return if (cities.isNotEmpty()) {
        this.copy(cities = cities)
    } else {
        null
    }
}

private fun RelayItem.Location.City.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    daita: Boolean,
    quic: Boolean,
    lwo: Boolean,
    ipVersion: Constraint<IpVersion>,
): RelayItem.Location.City? {
    val relays = relays.mapNotNull { it.filter(ownership, providers, daita, quic, lwo, ipVersion) }
    return if (relays.isNotEmpty()) {
        this.copy(relays = relays)
    } else {
        null
    }
}

private fun RelayItem.Location.Relay.requiredFeatures(
    requireDaita: Boolean,
    requireQuic: Boolean,
    requireLwo: Boolean,
    ipVersion: Constraint<IpVersion>,
): Boolean =
    when {
        // Can not require LWO and require QUIC at the same time
        requireDaita && requireQuic -> daita && quic?.supports(ipVersion) == true
        requireDaita && requireLwo -> daita && lwo
        requireDaita -> daita
        requireQuic -> quic?.supports(ipVersion) == true
        requireLwo -> lwo
        else -> true
    }

private fun Quic.supports(ipVersion: Constraint<IpVersion>) =
    when (ipVersion.getOrNull()) {
        IpVersion.IPV4 -> supportsIpv4
        IpVersion.IPV6 -> supportsIpv6
        else -> inAddresses.isNotEmpty()
    }

private fun RelayItem.Location.Relay.filter(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>,
    daita: Boolean,
    quic: Boolean,
    lwo: Boolean,
    ipVersion: Constraint<IpVersion>,
): RelayItem.Location.Relay? =
    if (
        requiredFeatures(daita, quic, lwo, ipVersion) &&
            hasOwnership(ownership) &&
            hasProvider(providers)
    )
        this
    else null

fun List<RelayItem.Location.Country>.findByGeoLocationId(
    geoLocationId: GeoLocationId
): RelayItem.Location? =
    when (geoLocationId) {
        is GeoLocationId.Country -> find { country -> country.id == geoLocationId }
        is GeoLocationId.City -> findCity(geoLocationId)
        is GeoLocationId.Hostname -> findRelay(geoLocationId)
    }

fun List<RelayItem.Location.Country>.findCity(
    geoLocationId: GeoLocationId.City
): RelayItem.Location.City? =
    find { country -> country.id == geoLocationId.country }
        ?.cities
        ?.find { city -> city.id == geoLocationId }

fun List<RelayItem.Location.Country>.findRelay(
    geoLocationId: GeoLocationId.Hostname
): RelayItem.Location.Relay? =
    find { country -> country.id == geoLocationId.country }
        ?.cities
        ?.find { city -> city.id == geoLocationId.city }
        ?.relays
        ?.find { relay -> relay.id == geoLocationId }

/**
 * Checks if two RelayItems are the same for the purpose of blocking selection. Only relays are
 * considered the same, cities and countries are not. For the purpose of blocking selection, custom
 * lists are considered to be a relay if and only if they contain a single relay.
 */
fun RelayItem.isTheSameAs(other: RelayItem): Boolean {
    return when (this) {
        is RelayItem.Location.Relay -> {
            (other is RelayItem.Location.Relay && this.id == other.id) ||
                (other is RelayItem.CustomList && other.onlyContains(this))
        }
        is RelayItem.CustomList -> {
            (other is RelayItem.Location.Relay && this.onlyContains(other)) ||
                (other is RelayItem.CustomList &&
                    this.locations.size == 1 &&
                    this.locations.first() is RelayItem.Location.Relay &&
                    other.onlyContains(this.locations.first()))
        }
        else -> false
    }
}
