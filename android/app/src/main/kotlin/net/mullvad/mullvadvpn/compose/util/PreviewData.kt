package net.mullvad.mullvadvpn.compose.util

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.relaylist.RelayItem

fun generateRelayItemCountry(
    name: String,
    cityNames: List<String>,
    relaysPerCity: Int,
    active: Boolean = true,
    expanded: Boolean = false,
    expandChildren: Boolean = false,
) =
    RelayItem.Country(
        name = name,
        code = name.generateCountryCode(),
        cities =
            cityNames.map { cityName ->
                generateRelayItemCity(
                    cityName,
                    name.generateCountryCode(),
                    relaysPerCity,
                    active,
                    expandChildren
                )
            },
        expanded = expanded,
    )

fun generateRelayItemCity(
    name: String,
    countryCode: String,
    numberOfRelays: Int,
    active: Boolean = true,
    expanded: Boolean = false,
) =
    RelayItem.City(
        name = name,
        code = name.generateCityCode(),
        relays =
            List(numberOfRelays) { index ->
                generateRelayItemRelay(
                    countryCode,
                    name.generateCityCode(),
                    generateHostname(countryCode, name.generateCityCode(), index),
                    active
                )
            },
        expanded = expanded,
        location = GeographicLocationConstraint.City(countryCode, name.generateCityCode()),
    )

fun generateRelayItemRelay(
    countryCode: String,
    cityCode: String,
    hostName: String,
    active: Boolean = true,
) =
    RelayItem.Relay(
        name = hostName,
        location =
            GeographicLocationConstraint.Hostname(
                countryCode = countryCode,
                cityCode = cityCode,
                hostname = hostName,
            ),
        locationName = "$cityCode $hostName",
        active = active
    )

private fun String.generateCountryCode() = (take(1) + takeLast(1)).lowercase()

private fun String.generateCityCode() = take(CITY_CODE_LENGTH).lowercase()

private fun generateHostname(countryCode: String, cityCode: String, index: Int) =
    "$countryCode-$cityCode-wg-${index+1}"

private const val CITY_CODE_LENGTH = 3
