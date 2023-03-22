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
                        val relayCity = RelayCity(relayCountry, city.name, city.code, false, relays)

                        val validCityRelays = city.relays.filter { relay -> relay.isWireguardRelay }

                        for (relay in validCityRelays) {
                            relays.add(Relay(relayCity, relay.hostname, relay.active))
                        }
                        relays.sortWith(RelayNameComparator)

                        if (relays.isNotEmpty()) {
                            cities.add(relayCity)
                        }
                    }

                    cities.sortBy({ it.name })
                    relayCountry
                }
                .filter { country -> country.cities.isNotEmpty() }
                .toMutableList()

        relayCountries.sortBy({ it.name })

        countries = relayCountries.toList()
    }

    fun findItemForLocation(
        constraint: Constraint<LocationConstraint>,
        expand: Boolean = false
    ): RelayItem? {
        when (constraint) {
            is Constraint.Any -> return null
            is Constraint.Only -> {
                val location = constraint.value

                when (location) {
                    is LocationConstraint.Country -> {
                        return countries.find { country -> country.code == location.countryCode }
                    }
                    is LocationConstraint.City -> {
                        val country =
                            countries.find { country -> country.code == location.countryCode }

                        if (expand) {
                            country?.expanded = true
                        }

                        return country?.cities?.find { city -> city.code == location.cityCode }
                    }
                    is LocationConstraint.Hostname -> {
                        val country =
                            countries.find { country -> country.code == location.countryCode }

                        val city = country?.cities?.find { city -> city.code == location.cityCode }

                        if (expand) {
                            country?.expanded = true
                            city?.expanded = true
                        }

                        return city?.relays?.find { relay -> relay.name == location.hostname }
                    }
                }
            }
        }
    }
}
