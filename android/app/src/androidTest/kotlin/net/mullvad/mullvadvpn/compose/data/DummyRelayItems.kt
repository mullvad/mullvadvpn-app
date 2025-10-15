package net.mullvad.mullvadvpn.compose.data

import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Quic
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayList
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData

private val DUMMY_RELAY_1 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo1"), "Relay City 1"),
                "Relay host 1",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = false,
        quic = null,
        lwo = false,
    )
private val DUMMY_RELAY_2 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo2"), "Relay City 2"),
                "Relay host 2",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = null,
        lwo = false,
    )
private val DUMMY_RELAY_3 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo3"), "Relay City 3"),
                "Relay host 3",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = true,
        quic = null,
        lwo = true,
    )
private val DUMMY_RELAY_4 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 4"),
                "Relay host 4",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = Quic(inAddresses = listOf()),
        lwo = true,
    )
private val DUMMY_RELAY_5 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 5",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = false,
        quic = Quic(inAddresses = listOf()),
        lwo = true,
    )
private val DUMMY_RELAY_6 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 6",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = true,
        quic = Quic(inAddresses = listOf()),
        lwo = true,
    )
private val DUMMY_RELAY_7 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 7",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = false,
        quic = Quic(inAddresses = listOf()),
        lwo = false,
    )
private val DUMMY_RELAY_8 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 8",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = false,
        quic = null,
        lwo = false,
    )
private val DUMMY_RELAY_9 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 9",
            ),
        active = true,
        provider = ProviderId("PROVIDER RENTED"),
        ownership = Ownership.Rented,
        daita = true,
        quic = null,
        lwo = true,
    )
private val DUMMY_RELAY_10 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 10",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = true,
        quic = null,
        lwo = false,
    )
private val DUMMY_RELAY_11 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 11",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = Quic(inAddresses = listOf()),
        lwo = false,
    )
private val DUMMY_RELAY_12 =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(GeoLocationId.Country("RCo4"), "Relay City 5"),
                "Relay host 12",
            ),
        active = true,
        provider = ProviderId("PROVIDER OWNED"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = Quic(inAddresses = listOf()),
        lwo = true,
    )
private val DUMMY_RELAY_CITY_1 =
    RelayItem.Location.City(
        name = "Relay City 1",
        id = GeoLocationId.City(country = GeoLocationId.Country("RCo1"), code = "RCi1"),
        relays = listOf(DUMMY_RELAY_1),
    )
private val DUMMY_RELAY_CITY_2 =
    RelayItem.Location.City(
        name = "Relay City 2",
        id = GeoLocationId.City(country = GeoLocationId.Country("RCo2"), code = "RCi2"),
        relays = listOf(DUMMY_RELAY_2),
    )
private val DUMMY_RELAY_CITY_3 =
    RelayItem.Location.City(
        name = "Relay City 3",
        id = GeoLocationId.City(country = GeoLocationId.Country("RCo3"), code = "RCi3"),
        relays = listOf(DUMMY_RELAY_3),
    )
private val DUMMY_RELAY_CITY_4 =
    RelayItem.Location.City(
        name = "Relay City 4",
        id = GeoLocationId.City(country = GeoLocationId.Country("RCo4"), code = "RCi4"),
        relays = listOf(DUMMY_RELAY_4),
    )
private val DUMMY_RELAY_CITY_5 =
    RelayItem.Location.City(
        name = "Relay City 5",
        id = GeoLocationId.City(country = GeoLocationId.Country("RCo4"), code = "RCi5"),
        relays =
            listOf(
                DUMMY_RELAY_5,
                DUMMY_RELAY_6,
                DUMMY_RELAY_7,
                DUMMY_RELAY_8,
                DUMMY_RELAY_9,
                DUMMY_RELAY_10,
                DUMMY_RELAY_11,
                DUMMY_RELAY_12,
            ),
    )
private val DUMMY_RELAY_COUNTRY_1 =
    RelayItem.Location.Country(
        name = "Relay Country 1",
        id = GeoLocationId.Country("RCo1"),
        cities = listOf(DUMMY_RELAY_CITY_1),
    )
private val DUMMY_RELAY_COUNTRY_2 =
    RelayItem.Location.Country(
        name = "Relay Country 2",
        id = GeoLocationId.Country("RCo2"),
        cities = listOf(DUMMY_RELAY_CITY_2),
    )
private val DUMMY_RELAY_COUNTRY_3 =
    RelayItem.Location.Country(
        name = "Relay Country 3",
        id = GeoLocationId.Country("RCo3"),
        cities = listOf(DUMMY_RELAY_CITY_3),
    )
private val DUMMY_RELAY_COUNTRY_4 =
    RelayItem.Location.Country(
        name = "Relay Country 4",
        id = GeoLocationId.Country("RCo4"),
        cities = listOf(DUMMY_RELAY_CITY_4, DUMMY_RELAY_CITY_5),
    )

private val DUMMY_WIREGUARD_PORT_RANGES = ArrayList<PortRange>()
private val DUMMY_SHADOWSOCKS_PORT_RANGES = emptyList<PortRange>()
private val DUMMY_WIREGUARD_ENDPOINT_DATA =
    WireguardEndpointData(DUMMY_WIREGUARD_PORT_RANGES, DUMMY_SHADOWSOCKS_PORT_RANGES)

val DUMMY_RELAY_COUNTRIES =
    listOf(
        DUMMY_RELAY_COUNTRY_1,
        DUMMY_RELAY_COUNTRY_2,
        DUMMY_RELAY_COUNTRY_3,
        DUMMY_RELAY_COUNTRY_4,
    )

val DUMMY_RELAY_LIST = RelayList(DUMMY_RELAY_COUNTRIES, DUMMY_WIREGUARD_ENDPOINT_DATA)

val DUMMY_RELAY_ITEM_CUSTOM_LISTS =
    listOf(
        RelayItem.CustomList(
            customList =
                CustomList(
                    name = CustomListName.fromString("First list"),
                    id = CustomListId("1"),
                    locations = emptyList(),
                ),
            locations = DUMMY_RELAY_COUNTRIES,
        ),
        RelayItem.CustomList(
            customList =
                CustomList(
                    name = CustomListName.fromString("Empty list"),
                    id = CustomListId("2"),
                    locations = emptyList(),
                ),
            locations = emptyList(),
        ),
    )

val DUMMY_CUSTOM_LISTS =
    listOf(
        CustomList(
            name = CustomListName.fromString("First list"),
            id = CustomListId("1"),
            locations = DUMMY_RELAY_COUNTRIES.map { it.id },
        ),
        CustomList(
            name = CustomListName.fromString("Empty list"),
            id = CustomListId("2"),
            locations = emptyList(),
        ),
    )
