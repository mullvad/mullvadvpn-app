package net.mullvad.mullvadvpn.compose.preview

import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem

internal object RelayItemPreviewData {
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
                        cityName,
                        name.generateCountryCode(),
                        relaysPerCity,
                        active,
                    )
                },
        )
}

private fun generateRelayItemCity(
    name: String,
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
                    name.generateCityCode(countryCode),
                    generateHostname(name.generateCityCode(countryCode), index),
                    active,
                )
            },
    )

private fun generateRelayItemRelay(
    cityCode: GeoLocationId.City,
    hostName: String,
    active: Boolean = true,
    daita: Boolean = true,
) =
    RelayItem.Location.Relay(
        id = GeoLocationId.Hostname(city = cityCode, code = hostName),
        active = active,
        provider = Provider(ProviderId("Provider"), Ownership.MullvadOwned),
        daita = daita,
    )

private fun String.generateCountryCode() =
    GeoLocationId.Country((take(1) + takeLast(1)).lowercase())

private fun String.generateCityCode(countryCode: GeoLocationId.Country) =
    GeoLocationId.City(countryCode, take(CITY_CODE_LENGTH).lowercase())

private fun generateHostname(city: GeoLocationId.City, index: Int) =
    "${city.country.code}-${city.code}-wg-${index+1}"

private const val CITY_CODE_LENGTH = 3
