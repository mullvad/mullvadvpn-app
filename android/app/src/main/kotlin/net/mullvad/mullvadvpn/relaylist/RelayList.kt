package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint

class RelayList {
    val countries: List<RelayCountry>

    constructor(model: net.mullvad.mullvadvpn.model.RelayList) {
        var relayCountries =
            model.countries
                .map { country ->
                    val cities = mutableListOf<RelayCity>()
                    val relayCountry = RelayCountry(country.name, country.code, false, cities)

                    for (city in country.cities) {
                        val relays = mutableListOf<Relay>()
                        val relayCity =
                            RelayCity(
                                name = city.name,
                                code = city.code,
                                location = LocationConstraint.City(country.code, city.code),
                                expanded = false,
                                relays = relays
                            )

                        val validCityRelays = city.relays.filter { relay -> relay.isWireguardRelay }

                        for (relay in validCityRelays) {
                            relays.add(
                                Relay(
                                    name = relay.hostname,
                                    location =
                                        LocationConstraint.Hostname(
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

        countries = relayCountries.toList()
    }

    constructor(countries: List<RelayCountry>) {
        this.countries = countries
    }

    fun findItemForLocation(constraint: Constraint<LocationConstraint>): RelayItem? {
        when (constraint) {
            is Constraint.Any -> return null
            is Constraint.Only -> {
                when (val location = constraint.value) {
                    is LocationConstraint.Country -> {
                        return countries.find { country -> country.code == location.countryCode }
                    }
                    is LocationConstraint.City -> {
                        val country =
                            countries.find { country -> country.code == location.countryCode }

                        return country?.cities?.find { city -> city.code == location.cityCode }
                    }
                    is LocationConstraint.Hostname -> {
                        val country =
                            countries.find { country -> country.code == location.countryCode }

                        val city = country?.cities?.find { city -> city.code == location.cityCode }

                        return city?.relays?.find { relay -> relay.name == location.hostname }
                    }
                }
            }
        }
    }

    fun filter(filter: String, selectedItem: RelayItem?): RelayList {
        return if (filter.length >= MIN_SEARCH_LENGTH) {
            val filteredCountries = mutableMapOf<String, RelayCountry>()
            countries.forEach { relayCountry ->
                val cities = mutableListOf<RelayCity>()
                if (relayCountry.name.contains(other = filter, ignoreCase = true)) {
                    filteredCountries[relayCountry.code] = relayCountry.copy(expanded = false)
                }

                relayCountry.cities.forEach { relayCity ->
                    val relays = mutableListOf<Relay>()
                    if (relayCity.name.contains(other = filter, ignoreCase = true)) {
                        if (filteredCountries.containsKey(relayCountry.code)) {
                            filteredCountries[relayCountry.code]?.expanded = true
                        } else {
                            filteredCountries[relayCountry.code] =
                                relayCountry.copy(expanded = true, cities = cities)
                        }
                        cities.add(relayCity.copy(expanded = false))
                    }

                    relayCity.relays.forEach { relay ->
                        if (relay.name.contains(other = filter, ignoreCase = true)) {
                            if (filteredCountries.containsKey(relayCountry.code)) {
                                filteredCountries[relayCountry.code]?.expanded = true
                            } else {
                                filteredCountries[relayCountry.code] =
                                    relayCountry.copy(expanded = true, cities = cities)
                            }
                            val city = cities.find { it.code == relayCity.code }
                            city?.let { city.expanded = true }
                                ?: run {
                                    cities.add(relayCity.copy(expanded = true, relays = relays))
                                }
                            relays.add(relay.copy())
                        }
                    }
                }
            }
            RelayList(filteredCountries.values.sortedBy { it.name })
        } else {
            this.expandItemForSelection(selectedItem)
        }
    }

    private fun expandItemForSelection(selectedItem: RelayItem?): RelayList {
        return selectedItem?.let {
            when (val location = selectedItem.location) {
                is LocationConstraint.Country -> {
                    return this
                }
                is LocationConstraint.City -> {
                    val country = countries.find { country -> country.code == location.countryCode }
                    country?.expanded = true

                    return this
                }
                is LocationConstraint.Hostname -> {
                    val country = countries.find { country -> country.code == location.countryCode }
                    val city = country?.cities?.find { city -> city.code == location.cityCode }

                    country?.expanded = true
                    city?.expanded = true

                    return this
                }
            }
        }
            ?: this
    }

    companion object {
        private const val MIN_SEARCH_LENGTH = 2
    }
}
