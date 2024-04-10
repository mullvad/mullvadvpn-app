package net.mullvad.mullvadvpn.compose.util

import java.util.UUID
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
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
    RelayItem.Location.City(
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
    RelayItem.Location.Relay(
        name = hostName,
        location =
            GeographicLocationConstraint.Hostname(
                countryCode = countryCode,
                cityCode = cityCode,
                hostname = hostName,
            ),
        locationName = "$cityCode $hostName",
        active = active,
        provider = "Provider",
        ownership = Ownership.MullvadOwned,
    )

private fun String.generateCountryCode() = (take(1) + takeLast(1)).lowercase()

private fun String.generateCityCode() = take(CITY_CODE_LENGTH).lowercase()

private fun generateHostname(countryCode: String, cityCode: String, index: Int) =
    "$countryCode-$cityCode-wg-${index+1}"

private const val CITY_CODE_LENGTH = 3

fun generateDevices(count: Int) = List(count) { generateDevice() }

fun generateDevice(
    id: DeviceId = DeviceId(UUID.randomUUID()),
    name: String? = null,
) =
    Device(
        id = id,
        name = name ?: "Device ${id.value.toString().take(DEVICE_SUFFIX_LENGTH)}",
        byteArrayOf(),
        DateTime.now()
    )

private const val DEVICE_SUFFIX_LENGTH = 4
