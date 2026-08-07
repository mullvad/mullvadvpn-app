package net.mullvad.mullvadvpn.lib.common.util.relaylist

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.NeedsOtherEntry
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.PartitionHostname
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

typealias RelayMetadataMap = Map<GeoLocationId.Hostname, RelayMetadata>
data class RelayMetadata(val needsOtherEntry: Boolean)
data class FilteredCountry(val country: RelayItem.Location.Country, val relayMetadata: RelayMetadataMap)
data class FilteredCity(val city: RelayItem.Location.City, val relayMetadata: RelayMetadataMap)
data class FilteredRelay(val relay: RelayItem.Location.Relay, val relayMetadata: RelayMetadata)

fun RelayItem.Location.Country.filter(
    validHostnames: Map<PartitionHostname, NeedsOtherEntry>
): FilteredCountry? {
    val cities = cities.mapNotNull { it.filter(validHostnames) }

    return if (cities.isNotEmpty()) {
        val cityItems = cities.map { it.city }
        val metadata = buildMap {
            cities.map(FilteredCity::relayMetadata).forEach(::putAll)
        }
        FilteredCountry(
            country = this.copy(cities = cityItems),
            relayMetadata = metadata
        )
    } else {
        null
    }
}

private fun RelayItem.Location.City.filter(
    validHostnames: Map<PartitionHostname, NeedsOtherEntry>
): FilteredCity? {
    val relays = relays.mapNotNull { it.filter(validHostnames) }

    return if (relays.isNotEmpty()) {
        val relayItems = relays.map { it.relay }
        val metadata = relays.associate { it.relay.id to it.relayMetadata }
        FilteredCity(
            city = this.copy(relays = relayItems),
            relayMetadata = metadata
        )
    } else {
        null
    }
}

private fun RelayItem.Location.Relay.filter(
    validHostnames: Map<PartitionHostname, NeedsOtherEntry>
): FilteredRelay? {
    // If host name is not in validHostnames return null
    val needsOtherEntry = validHostnames[id.code] ?: return null
    return FilteredRelay(relay = this, relayMetadata = RelayMetadata(needsOtherEntry = needsOtherEntry))
}

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
): RelayItem.Location.City? = find { country ->
    country.id == geoLocationId.country
}
    ?.cities
    ?.find { city -> city.id == geoLocationId }

fun List<RelayItem.Location.Country>.findRelay(
    geoLocationId: GeoLocationId.Hostname
): RelayItem.Location.Relay? = find { country ->
    country.id == geoLocationId.country
}
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
