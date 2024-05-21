package net.mullvad.mullvadvpn.compose.data

import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayList
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData

private val DUMMY_RELAY_1 =
    RelayItem.Location.Relay(
        id =
            net.mullvad.mullvadvpn.lib.model.GeoLocationId.Hostname(
                city = net.mullvad.mullvadvpn.lib.model.GeoLocationId.City(net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo1"), "Relay City 1"),
                "Relay host 1"
            ),
        active = true,
        provider =
            Provider(
                providerId = ProviderId("PROVIDER RENTED"),
                ownership = Ownership.Rented,
            )
    )
private val DUMMY_RELAY_2 =
    RelayItem.Location.Relay(
        id =
            net.mullvad.mullvadvpn.lib.model.GeoLocationId.Hostname(
                city = net.mullvad.mullvadvpn.lib.model.GeoLocationId.City(net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo2"), "Relay City 2"),
                "Relay host 2"
            ),
        active = true,
        provider =
            Provider(providerId = ProviderId("PROVIDER OWNED"), ownership = Ownership.MullvadOwned)
    )
private val DUMMY_RELAY_CITY_1 =
    RelayItem.Location.City(
        name = "Relay City 1",
        id = net.mullvad.mullvadvpn.lib.model.GeoLocationId.City(countryCode = net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo1"), cityCode = "RCi1"),
        relays = listOf(DUMMY_RELAY_1),
        expanded = false
    )
private val DUMMY_RELAY_CITY_2 =
    RelayItem.Location.City(
        name = "Relay City 2",
        id = net.mullvad.mullvadvpn.lib.model.GeoLocationId.City(countryCode = net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo2"), cityCode = "RCi2"),
        relays = listOf(DUMMY_RELAY_2),
        expanded = false
    )
private val DUMMY_RELAY_COUNTRY_1 =
    RelayItem.Location.Country(
        name = "Relay Country 1",
        id = net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo1"),
        expanded = false,
        cities = listOf(DUMMY_RELAY_CITY_1)
    )
private val DUMMY_RELAY_COUNTRY_2 =
    RelayItem.Location.Country(
        name = "Relay Country 2",
        id = net.mullvad.mullvadvpn.lib.model.GeoLocationId.Country("RCo2"),
        expanded = false,
        cities = listOf(DUMMY_RELAY_CITY_2)
    )

private val DUMMY_WIREGUARD_PORT_RANGES = ArrayList<PortRange>()
private val DUMMY_WIREGUARD_ENDPOINT_DATA = WireguardEndpointData(DUMMY_WIREGUARD_PORT_RANGES)

val DUMMY_RELAY_COUNTRIES = listOf(DUMMY_RELAY_COUNTRY_1, DUMMY_RELAY_COUNTRY_2)

val DUMMY_RELAY_LIST =
    net.mullvad.mullvadvpn.lib.model.RelayList(
        DUMMY_RELAY_COUNTRIES,
        DUMMY_WIREGUARD_ENDPOINT_DATA,
    )

val DUMMY_RELAY_ITEM_CUSTOM_LISTS =
    listOf(
        RelayItem.CustomList(
            customListName = CustomListName.fromString("First list"),
            expanded = false,
            id = net.mullvad.mullvadvpn.lib.model.CustomListId("1"),
            locations = DUMMY_RELAY_COUNTRIES
        ),
        RelayItem.CustomList(
            customListName = CustomListName.fromString("Empty list"),
            expanded = false,
            id = net.mullvad.mullvadvpn.lib.model.CustomListId("2"),
            locations = emptyList()
        )
    )

val DUMMY_CUSTOM_LISTS =
    listOf(
        CustomList(
            name = CustomListName.fromString("First list"),
            id = net.mullvad.mullvadvpn.lib.model.CustomListId("1"),
            locations = DUMMY_RELAY_COUNTRIES.map { it.id }
        ),
        CustomList(
            name = CustomListName.fromString("Empty list"),
            id = net.mullvad.mullvadvpn.lib.model.CustomListId("2"),
            locations = emptyList()
        )
    )
