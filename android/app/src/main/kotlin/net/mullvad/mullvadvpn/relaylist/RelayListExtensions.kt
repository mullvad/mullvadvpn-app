package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.Relay as DaemonRelay
import net.mullvad.mullvadvpn.model.RelayList

/**
 * Convert from a model.RelayList to list of relaylist.RelayCountry Non-wiregaurd relays are
 * filtered out and also relays that do not fit the ownership and provider list So are also cities
 * that only contains non-wireguard relays Countries, cities and relays are ordered by name
 */
fun RelayList.toRelayCountries(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): List<RelayItem.Country> {
    val relayCountries =
        this.countries
            .map { country ->
                val cities = mutableListOf<RelayItem.City>()
                val relayCountry = RelayItem.Country(country.name, country.code, false, cities)

                for (city in country.cities) {
                    val relays = mutableListOf<RelayItem.Relay>()
                    val relayCity =
                        RelayItem.City(
                            name = city.name,
                            code = city.code,
                            location = GeographicLocationConstraint.City(country.code, city.code),
                            expanded = false,
                            relays = relays
                        )

                    val validCityRelays =
                        city.relays.filterValidRelays(ownership = ownership, providers = providers)

                    for (relay in validCityRelays) {
                        relays.add(
                            RelayItem.Relay(
                                name = relay.hostname,
                                location =
                                    GeographicLocationConstraint.Hostname(
                                        country.code,
                                        city.code,
                                        relay.hostname
                                    ),
                                locationName = "${city.name} (${relay.hostname})",
                                active = relay.active
                            )
                        )
                    }
                    relays.sortWith(RelayNameComparator)

                    if (relays.isNotEmpty()) {
                        cities.add(relayCity)
                    }
                }

                cities.sortBy { it.name }
                relayCountry
            }
            .filter { country -> country.cities.isNotEmpty() }
            .toMutableList()

    relayCountries.sortBy { it.name }

    return relayCountries.toList()
}

fun List<RelayItem.Country>.findItemForGeographicLocationConstraint(
    constraint: GeographicLocationConstraint
) =
    when (constraint) {
        is GeographicLocationConstraint.Country -> {
            this.find { country -> country.code == constraint.countryCode }
        }
        is GeographicLocationConstraint.City -> {
            val country = this.find { country -> country.code == constraint.countryCode }

            country?.cities?.find { city -> city.code == constraint.cityCode }
        }
        is GeographicLocationConstraint.Hostname -> {
            val country = this.find { country -> country.code == constraint.countryCode }

            val city = country?.cities?.find { city -> city.code == constraint.cityCode }

            city?.relays?.find { relay -> relay.name == constraint.hostname }
        }
    }

/**
 * Filter and expand the list based on search terms If a country is matched, that country and all
 * its children are added to the list, but the country is not expanded If a city is matched, its
 * parent country is added and expanded if needed and its children are added, but the city is not
 * expanded If a relay is matched, its parents are added and expanded and itself is also added.
 */
@Suppress("NestedBlockDepth")
fun List<RelayItem.Country>.filterOnSearchTerm(
    searchTerm: String,
    selectedItem: RelayItem?
): List<RelayItem.Country> {
    return if (searchTerm.length >= MIN_SEARCH_LENGTH) {
        val filteredCountries = mutableMapOf<String, RelayItem.Country>()
        this.forEach { relayCountry ->
            val cities = mutableListOf<RelayItem.City>()

            // Try to match the search term with a country
            // If we match a country, add that country and all cities and relays in that country
            // Do not currently expand the country or any city
            if (relayCountry.name.contains(other = searchTerm, ignoreCase = true)) {
                cities.addAll(relayCountry.cities.map { city -> city.copy(expanded = false) })
                filteredCountries[relayCountry.code] =
                    relayCountry.copy(expanded = false, cities = cities)
            }

            // Go through and try to match the search term with every city
            relayCountry.cities.forEach { relayCity ->
                val relays = mutableListOf<RelayItem.Relay>()
                // If we match and we already added the country to the filtered list just expand the
                // country.
                // If the country is not currently in the filtered list, add it and expand it.
                // Finally if the city has not already been added to the filtered list, add it, but
                // do not expand it yet.
                if (relayCity.name.contains(other = searchTerm, ignoreCase = true)) {
                    val value = filteredCountries[relayCountry.code]
                    if (value != null) {
                        filteredCountries[relayCountry.code] = value.copy(expanded = true)
                    } else {
                        filteredCountries[relayCountry.code] =
                            relayCountry.copy(expanded = true, cities = cities)
                    }
                    if (cities.none { city -> city.code == relayCity.code }) {
                        cities.add(relayCity.copy(expanded = false))
                    }
                }

                // Go through and try to match the search term with every relay
                relayCity.relays.forEach { relay ->
                    // If we match a relay, check if the county the relay is in already is added, if
                    // so expand, if not add and expand the country.
                    // Check if the city that the relay is in is already added to the filtered list,
                    // if so expand it, if not add it to the filtered list and expand it.
                    // Finally add the relay to the list.
                    if (relay.name.contains(other = searchTerm, ignoreCase = true)) {
                        val value = filteredCountries[relayCountry.code]
                        if (value != null) {
                            filteredCountries[relayCountry.code] = value.copy(expanded = true)
                        } else {
                            filteredCountries[relayCountry.code] =
                                relayCountry.copy(expanded = true, cities = cities)
                        }
                        val cityIndex = cities.indexOfFirst { it.code == relayCity.code }

                        // No city found
                        if (cityIndex < 0) {
                            cities.add(relayCity.copy(expanded = true, relays = relays))
                        } else {
                            // Update found city as expanded
                            cities[cityIndex] = cities[cityIndex].copy(expanded = true)
                        }

                        relays.add(relay.copy())
                    }
                }
            }
        }
        filteredCountries.values.sortedBy { it.name }
    } else {
        this.expandItemForSelection(selectedItem)
    }
}

private fun List<DaemonRelay>.filterValidRelays(
    ownership: Constraint<Ownership>,
    providers: Constraint<Providers>
): List<DaemonRelay> =
    filter { it.isWireguardRelay }
        .filter {
            when (ownership) {
                is Constraint.Any -> true
                is Constraint.Only ->
                    when (ownership.value) {
                        Ownership.MullvadOwned -> it.owned
                        Ownership.Rented -> !it.owned
                    }
            }
        }
        .filter { relay ->
            when (providers) {
                is Constraint.Any -> true
                is Constraint.Only -> providers.value.providers.contains(relay.provider)
            }
        }

/** Expand the parent(s), if any, for the current selected item */
private fun List<RelayItem.Country>.expandItemForSelection(
    selectedItem: RelayItem?
): List<RelayItem.Country> {
    return selectedItem?.let {
        when (selectedItem) {
            is RelayItem.Country -> {
                this
            }
            is RelayItem.City -> {
                this.map { country ->
                    if (country.code == selectedItem.location.countryCode) {
                        country.copy(expanded = true)
                    } else {
                        country
                    }
                }
            }
            is RelayItem.Relay -> {
                this.map { country ->
                    if (country.code == selectedItem.location.countryCode) {
                        country.copy(
                            expanded = true,
                            cities =
                                country.cities.map { city ->
                                    if (city.code == selectedItem.location.cityCode) {
                                        city.copy(expanded = true)
                                    } else {
                                        city
                                    }
                                }
                        )
                    } else {
                        country
                    }
                }
            }
            is RelayItem.CustomList -> this
        }
    } ?: this
}

private const val MIN_SEARCH_LENGTH = 2
