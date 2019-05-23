package net.mullvad.mullvadvpn.relaylist

class RelayList {
    val countries: List<RelayCountry>

    constructor(model: net.mullvad.mullvadvpn.model.RelayList) {
        countries = model.countries.map { country ->
            val cities = country.cities.map { city -> 
                val relays = city.relays.map { relay -> Relay(relay.hostname) }

                RelayCity(city.name, city.code, false, relays)
            }

            RelayCountry(country.name, country.code, false, cities)
        }
    }
}
