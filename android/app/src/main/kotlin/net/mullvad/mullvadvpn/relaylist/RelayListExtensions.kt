package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId) =
    withDescendants().firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId.City) =
    flatMap { it.cities }.firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.search(searchTerm: String): List<GeoLocationId> =
    withDescendants().filter { it.name.contains(searchTerm, ignoreCase = true) }.map { it.id }

fun List<GeoLocationId>.expansionSet() = flatMap { it.ancestors() }.toSet()

fun List<RelayItem.Location.Country>.newFilterOnSearch(
    searchTerm: String
): Pair<Set<GeoLocationId>, List<RelayItem.Location.Country>> {
    val matchesIds = search(searchTerm)
    val expansionSet = matchesIds.expansionSet()

    val filteredCountryList = mapNotNull { country ->
        if (country.id in matchesIds) {
            country
        } else if (country.id in expansionSet) {
            country.copy(
                cities =
                    country.cities.mapNotNull { city ->
                        if (city.id in matchesIds) {
                            city
                        } else if (city.id in expansionSet) {
                            city.copy(
                                relays = city.relays.filter { relay -> relay.id in matchesIds }
                            )
                        } else null
                    }
            )
        } else {
            null
        }
    }
    return expansionSet to filteredCountryList
}

fun GeoLocationId.ancestors(): List<GeoLocationId> =
    when (this) {
        is GeoLocationId.City -> listOf(country)
        is GeoLocationId.Country -> emptyList()
        is GeoLocationId.Hostname -> listOf(country, city)
    }

fun List<RelayItem.Location.Country>.getRelayItemsByCodes(
    codes: List<GeoLocationId>
): List<RelayItem.Location> =
    this.filter { codes.contains(it.id) } +
        this.flatMap { it.descendants() }.filter { codes.contains(it.id) }

fun <T : RelayItem> List<T>.sortedByName() =
    this.sortedWith(compareBy(String.CASE_INSENSITIVE_ORDER) { it.name })
