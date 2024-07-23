package net.mullvad.mullvadvpn.relaylist

import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId) =
    withDescendants().firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId.City) =
    flatMap { it.cities }.firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.newFilterOnSearch(searchTerm: String): Pair<Set<GeoLocationId>, List<RelayItem.Location.Country>> {
    val matchesIds =
        withDescendants().filter { it.name.contains(searchTerm, ignoreCase = true) }.map { it.id }

    val expansionSet = matchesIds.flatMap { it.parents() }.toSet()
    Logger.d("Expansion Set: $expansionSet")

    val filteredCountryList = filter { it.id in expansionSet || it.id in matchesIds }
        .map {
            it.copy(
                cities =
                    it.cities
                        .filter {
                            it.id in expansionSet ||
                                it.id in matchesIds ||
                                it.id.country in matchesIds
                        }
                        .map {
                            it.copy(
                                relays =
                                    it.relays.filter {
                                        it.id in expansionSet ||
                                            it.id in matchesIds ||
                                            it.id.city in matchesIds ||
                                            it.id.country in matchesIds
                                    })
                        })
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
