package net.mullvad.mullvadvpn.relaylist

import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId) =
    withDescendants().firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId.City) =
    flatMap { it.cities }.firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.newFilterOnSearch(
    searchTerm: String
): Pair<Set<GeoLocationId>, List<RelayItem.Location.Country>> {
    val matchesIds =
        withDescendants().filter { it.name.contains(searchTerm, ignoreCase = true) }.map { it.id }

    val expansionSet = matchesIds.flatMap { it.parents() }.toSet()
    Logger.d("Expansion Set: $expansionSet")

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
                                relays = city.relays.filter { relay -> relay.id in matchesIds })
                        } else null
                    })
        } else {
            null
        }
    }
    return expansionSet to filteredCountryList
}

private fun GeoLocationId.parents(): List<GeoLocationId> =
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
