package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem

fun generateRelayItemCountry(
    name: String,
    cityNames: List<String>,
    relaysPerCity: Int,
    active: Boolean = true,
) =
    RelayItem.Location.Country(
        name = name,
        id = name.generateCountryCode(),
        cities =
            cityNames.map { cityName ->
                generateRelayItemCity(
                    name = cityName,
                    countryName = name,
                    countryCode = name.generateCountryCode(),
                    numberOfRelays = relaysPerCity,
                    active = active,
                )
            },
    )

private fun generateRelayItemCity(
    name: String,
    countryName: String,
    countryCode: GeoLocationId.Country,
    numberOfRelays: Int,
    active: Boolean = true,
) =
    RelayItem.Location.City(
        name = name,
        id = name.generateCityCode(countryCode),
        relays =
            List(numberOfRelays) { index ->
                generateRelayItemRelay(
                    cityCode = name.generateCityCode(countryCode),
                    hostName = generateHostname(name.generateCityCode(countryCode), index),
                    active = active,
                    cityName = name,
                    countryName = countryName,
                )
            },
        countryName = countryName,
    )

private fun generateRelayItemRelay(
    cityCode: GeoLocationId.City,
    hostName: String,
    cityName: String,
    countryName: String,
    active: Boolean = true,
    daita: Boolean = true,
) =
    RelayItem.Location.Relay(
        id = GeoLocationId.Hostname(city = cityCode, code = hostName),
        active = active,
        provider = ProviderId("Provider"),
        ownership = Ownership.MullvadOwned,
        daita = daita,
        quic = null,
        lwo = false,
        cityName = cityName,
        countryName = countryName,
    )

private fun String.generateCountryCode() =
    GeoLocationId.Country((take(1) + takeLast(1)).lowercase())

private fun String.generateCityCode(countryCode: GeoLocationId.Country) =
    GeoLocationId.City(countryCode, take(CITY_CODE_LENGTH).lowercase())

private fun generateHostname(city: GeoLocationId.City, index: Int) =
    "${city.country.code}-${city.code}-wg-${index+1}"

private const val CITY_CODE_LENGTH = 3
