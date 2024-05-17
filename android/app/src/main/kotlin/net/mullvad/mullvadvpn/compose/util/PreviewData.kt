package net.mullvad.mullvadvpn.compose.util

import java.util.UUID
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.ProviderId
import net.mullvad.mullvadvpn.model.RelayItem
import org.joda.time.DateTime

fun generateRelayItemCountry(
    name: String,
    cityNames: List<String>,
    relaysPerCity: Int,
    active: Boolean = true,
    expanded: Boolean = false,
    expandChildren: Boolean = false,
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
                    expandChildren
                )
            },
        expanded = expanded,
    )

fun generateRelayItemCity(
    name: String,
    countryCode: GeoLocationId.Country,
    numberOfRelays: Int,
    active: Boolean = true,
    expanded: Boolean = false,
) =
    RelayItem.Location.City(
        name = name,
        id = name.generateCityCode(countryCode),
        relays =
            List(numberOfRelays) { index ->
                generateRelayItemRelay(
                    name.generateCityCode(countryCode),
                    generateHostname(name.generateCityCode(countryCode), index),
                    active
                )
            },
        expanded = expanded,
    )

fun generateRelayItemRelay(
    cityCode: GeoLocationId.City,
    hostName: String,
    active: Boolean = true,
) =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = cityCode,
                hostname = hostName,
            ),
        active = active,
        provider = Provider(ProviderId("Provider"), Ownership.MullvadOwned),
    )

private fun String.generateCountryCode() =
    GeoLocationId.Country((take(1) + takeLast(1)).lowercase())

private fun String.generateCityCode(countryCode: GeoLocationId.Country) =
    GeoLocationId.City(countryCode, take(CITY_CODE_LENGTH).lowercase())

private fun generateHostname(city: GeoLocationId.City, index: Int) =
    "${city.countryCode.countryCode}-${city.cityCode}-wg-${index+1}"

private const val CITY_CODE_LENGTH = 3

fun generateDevices(count: Int) = List(count) { generateDevice() }

fun generateDevice(
    id: DeviceId = DeviceId(UUID.randomUUID()),
    name: String? = null,
) =
    Device(
        id = id,
        name = name ?: "Device ${id.value.toString().take(DEVICE_SUFFIX_LENGTH)}",
        DateTime.now()
    )

private const val DEVICE_SUFFIX_LENGTH = 4
