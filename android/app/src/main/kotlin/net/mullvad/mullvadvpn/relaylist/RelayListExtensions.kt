package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.RelayList

fun RelayList.toRelayCountries(): List<RelayCountry> {
    val relayCountries =
        this.countries
            .map { country ->
                val cities = mutableListOf<RelayCity>()
                val relayCountry = RelayCountry(country.name, country.code, false, cities)

                for (city in country.cities) {
                    val relays = mutableListOf<Relay>()
                    val relayCity =
                        RelayCity(
                            name = city.name,
                            code = city.code,
                            location = GeographicLocationConstraint.City(country.code, city.code),
                            expanded = false,
                            relays = relays
                        )

                    val validCityRelays = city.relays.filter { relay -> relay.isWireguardRelay }

                    for (relay in validCityRelays) {
                        relays.add(
                            Relay(
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

fun List<RelayCountry>.findItemForLocation(
    constraint: Constraint<GeographicLocationConstraint>
): RelayItem? {
    when (constraint) {
        is Constraint.Any -> return null
        is Constraint.Only -> {
            when (val location = constraint.value) {
                is GeographicLocationConstraint.Country -> {
                    return this.find { country -> country.code == location.countryCode }
                }
                is GeographicLocationConstraint.City -> {
                    val country = this.find { country -> country.code == location.countryCode }

                    return country?.cities?.find { city -> city.code == location.cityCode }
                }
                is GeographicLocationConstraint.Hostname -> {
                    val country = this.find { country -> country.code == location.countryCode }

                    val city = country?.cities?.find { city -> city.code == location.cityCode }

                    return city?.relays?.find { relay -> relay.name == location.hostname }
                }
            }
        }
    }
}

fun List<RelayCountry>.filterOnSearchTerm(
    searchTerm: String,
    selectedItem: RelayItem?
): List<RelayCountry> {
    return if (searchTerm.length >= MIN_SEARCH_LENGTH) {
        val filteredCountries = mutableMapOf<String, RelayCountry>()
        this.forEach { relayCountry ->
            val cities = mutableListOf<RelayCity>()
            if (relayCountry.name.contains(other = searchTerm, ignoreCase = true)) {
                cities.addAll(relayCountry.cities.map { city -> city.copy(expanded = false) })
                filteredCountries[relayCountry.code] =
                    relayCountry.copy(expanded = false, cities = cities)
            }

            relayCountry.cities.forEach { relayCity ->
                val relays = mutableListOf<Relay>()
                if (relayCity.name.contains(other = searchTerm, ignoreCase = true)) {
                    if (filteredCountries.containsKey(relayCountry.code)) {
                        filteredCountries[relayCountry.code]?.expanded = true
                    } else {
                        filteredCountries[relayCountry.code] =
                            relayCountry.copy(expanded = true, cities = cities)
                    }
                    if (cities.none { city -> city.code == relayCity.code }) {
                        cities.add(relayCity.copy(expanded = false))
                    }
                }

                relayCity.relays.forEach { relay ->
                    if (relay.name.contains(other = searchTerm, ignoreCase = true)) {
                        if (filteredCountries.containsKey(relayCountry.code)) {
                            filteredCountries[relayCountry.code]?.expanded = true
                        } else {
                            filteredCountries[relayCountry.code] =
                                relayCountry.copy(expanded = true, cities = cities)
                        }
                        val city = cities.find { it.code == relayCity.code }
                        city?.let { city.expanded = true }
                            ?: run { cities.add(relayCity.copy(expanded = true, relays = relays)) }
                        relays.add(relay.copy())
                    }
                }
            }
        }
        return filteredCountries.values.sortedBy { it.name }
    } else {
        return this.expandItemForSelection(selectedItem)
    }
}

fun List<RelayCountry>.expandItemForSelection(selectedItem: RelayItem?): List<RelayCountry> {
    return selectedItem?.let {
        when (val location = selectedItem.location) {
            is GeographicLocationConstraint.Country -> {
                return this
            }
            is GeographicLocationConstraint.City -> {
                return this.map { country ->
                    if (country.code == location.countryCode) {
                        country.copy(expanded = true)
                    } else {
                        country
                    }
                }
            }
            is GeographicLocationConstraint.Hostname -> {
                return this.map { country ->
                    if (country.code == location.countryCode) {
                        country.copy(
                            expanded = true,
                            cities =
                                country.cities.map { city ->
                                    if (city.code == location.cityCode) {
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
        }
    }
        ?: this
}

private const val MIN_SEARCH_LENGTH = 2
