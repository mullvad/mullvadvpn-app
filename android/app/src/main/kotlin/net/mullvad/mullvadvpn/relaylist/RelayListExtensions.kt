package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId) =
    withDescendants().firstOrNull { it.id == geoLocationId }

fun List<RelayItem.Location.Country>.findByGeoLocationId(geoLocationId: GeoLocationId.City) =
    flatMap { it.cities }.firstOrNull { it.id == geoLocationId }

/**
 * Filter and expand the list based on search terms If a country is matched, that country and all
 * its children are added to the list, but the country is not expanded If a city is matched, its
 * parent country is added and expanded if needed and its children are added, but the city is not
 * expanded If a relay is matched, its parents are added and expanded and itself is also added.
 */
@Suppress("NestedBlockDepth")
// fun List<RelayItem.Location.Country>.filterOnSearchTerm(
//    searchTerm: String,
//    selectedItem: RelayItemId?
// ): List<RelayItem.Location.Country> {
//    return if (searchTerm.length >= MIN_SEARCH_LENGTH) {
//        val filteredCountries = mutableMapOf<GeoLocationId.Country, RelayItem.Location.Country>()
//        this.forEach { relayCountry ->
//            val cities = mutableListOf<RelayItem.Location.City>()
//
//            // Try to match the search term with a country
//            // If we match a country, add that country and all cities and relays in that country
//            // Do not currently expand the country or any city
//            if (relayCountry.name.contains(other = searchTerm, ignoreCase = true)) {
//                cities.addAll(relayCountry.cities.map { city -> city.copy(expanded = false) })
//                filteredCountries[relayCountry.id] =
//                    relayCountry.copy(expanded = false, cities = cities)
//            }
//
//            // Go through and try to match the search term with every city
//            relayCountry.cities.forEach { relayCity ->
//                val relays = mutableListOf<RelayItem.Location.Relay>()
//                // If we match and we already added the country to the filtered list just expand
// the
//                // country.
//                // If the country is not currently in the filtered list, add it and expand it.
//                // Finally if the city has not already been added to the filtered list, add it,
// but
//                // do not expand it yet.
//                if (relayCity.name.contains(other = searchTerm, ignoreCase = true)) {
//                    val value = filteredCountries[relayCountry.id]
//                    if (value != null) {
//                        filteredCountries[relayCountry.id] = value.copy(expanded = true)
//                    } else {
//                        filteredCountries[relayCountry.id] =
//                            relayCountry.copy(expanded = true, cities = cities)
//                    }
//                    if (cities.none { city -> city.id == relayCity.id }) {
//                        cities.add(relayCity.copy(expanded = false))
//                    }
//                }
//
//                // Go through and try to match the search term with every relay
//                relayCity.relays.forEach { relay ->
//                    // If we match a relay, check if the county the relay is in already is added,
// if
//                    // so expand, if not add and expand the country.
//                    // Check if the city that the relay is in is already added to the filtered
// list,
//                    // if so expand it, if not add it to the filtered list and expand it.
//                    // Finally add the relay to the list.
//                    if (relay.name.contains(other = searchTerm, ignoreCase = true)) {
//                        val value = filteredCountries[relayCountry.id]
//                        if (value != null) {
//                            filteredCountries[relayCountry.id] = value.copy(expanded = true)
//                        } else {
//                            filteredCountries[relayCountry.id] =
//                                relayCountry.copy(expanded = true, cities = cities)
//                        }
//                        val cityIndex = cities.indexOfFirst { it.id == relayCity.id }
//
//                        // No city found
//                        if (cityIndex < 0) {
//                            cities.add(relayCity.copy(expanded = true, relays = relays))
//                        } else {
//                            // Update found city as expanded
//                            cities[cityIndex] = cities[cityIndex].copy(expanded = true)
//                        }
//
//                        relays.add(relay.copy())
//                    }
//                }
//            }
//        }
//        filteredCountries.values.sortedBy { it.name }
//    } else {
//        this.expandItemForSelection(selectedItem)
//    }
// }

/// ** Expand the parent(s), if any, for the current selected item */
// private fun List<RelayItem.Location.Country>.expandItemForSelection(
//    selectedItem: RelayItemId?
// ): List<RelayItem.Location.Country> {
//    selectedItem ?: return this
//    return when (selectedItem) {
//        is CustomListId,
//        is GeoLocationId.Country -> this
//        is GeoLocationId.City ->
//            map { if (it.id == selectedItem.country) it.copy(expanded = true) else it }
//        is GeoLocationId.Hostname -> {
//            map { country ->
//                if (country.id == selectedItem.country) {
//                    country.copy(
//                        expanded = true,
//                        cities =
//                            country.cities.map { city ->
//                                if (city.id == selectedItem.city) {
//                                    city.copy(expanded = true)
//                                } else {
//                                    city
//                                }
//                            },
//                    )
//                } else {
//                    country
//                }
//            }
//        }
//    }
// }

fun List<RelayItem.Location.Country>.getRelayItemsByCodes(
    codes: List<GeoLocationId>
): List<RelayItem.Location> =
    this.filter { codes.contains(it.id) } +
        this.flatMap { it.descendants() }.filter { codes.contains(it.id) }
