package net.mullvad.mullvadvpn.feature.location.impl.data

import io.mockk.mockk
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DomainCustomList
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem

val RELAY_COUNTRY_SWEDEN =
    RelayItem.Location.Country(
        id = GeoLocationId.Country("se"),
        name = "Sweden",
        cities =
            listOf(
                RelayItem.Location.City(
                    id = GeoLocationId.City(GeoLocationId.Country("se"), "got"),
                    name = "Gothenburg",
                    latLong = LatLong(0f, 0f),
                    countryName = "Sweden",
                    relays =
                        listOf(
                            RelayItem.Location.Relay(
                                id =
                                    GeoLocationId.Hostname(
                                        GeoLocationId.City(GeoLocationId.Country("se"), "got"),
                                        "wg-got-001",
                                    ),
                                active = true,
                                latLong = mockk(relaxed = true),
                                ownership = mockk(relaxed = true),
                                provider = mockk(relaxed = true),
                                countryName = "Sweden",
                                cityName = "Gothenburg",
                                daita = false,
                                quic = null,
                                lwo = false,
                            )
                        ),
                )
            ),
    )

val RELAY_COUNTRY_NORWAY =
    RelayItem.Location.Country(id = GeoLocationId.Country("no"), "Norway", emptyList())

val RELAY_ID_BE_BRU_WG_101 =
    GeoLocationId.Hostname(
        code = "be-bru-wg-101",
        city =
            GeoLocationId.City(
                code = "bru",
                country = GeoLocationId.Country(code = "be"),
            ),
    )

val RELAY_BE_BRU_WG_101 =
    RelayItem.Location.Relay(
        id = RELAY_ID_BE_BRU_WG_101,
        active = true,
        latLong = mockk(relaxed = true),
        provider = ProviderId("mullvad"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = null,
        lwo = false,
        cityName = "Brussels",
        countryName = "Belgium",
    )

val RELAY_ID_BE_BRU_WG_102 =
    GeoLocationId.Hostname(
        code = "be-bru-wg-102",
        city =
            GeoLocationId.City(
                code = "bru",
                country = GeoLocationId.Country(code = "be"),
            ),
    )

val RELAY_BE_BRU_WG_102 =
    RelayItem.Location.Relay(
        id = RELAY_ID_BE_BRU_WG_102,
        active = true,
        latLong = mockk(relaxed = true),
        provider = ProviderId("mullvad"),
        ownership = Ownership.MullvadOwned,
        daita = false,
        quic = null,
        lwo = false,
        cityName = "Brussels",
        countryName = "Belgium",
    )

val RELAY_COUNTRY_BELGIUM =
    RelayItem.Location.Country(
        id = GeoLocationId.Country("be"),
        name = "Belgium - Brussels",
        cities =
            listOf(
                RelayItem.Location.City(
                    id =
                        GeoLocationId.City(
                            code = "bru",
                            country = GeoLocationId.Country(code = "be"),
                        ),
                    name = "Brussels",
                    latLong = mockk(relaxed = true),
                    countryName = "Belgium",
                    relays =
                        listOf(
                            RELAY_BE_BRU_WG_101,
                            RELAY_BE_BRU_WG_102,
                        ),
                )
            ),
    )

val CUSTOM_LIST_ID_AA = CustomListId("aa")

val CUSTOM_LIST_AA =
    RelayItem.CustomList(
        customList =
            DomainCustomList(
                id = CUSTOM_LIST_ID_AA,
                name = CustomListName.fromString("Custom List AA"),
                locations = listOf(RELAY_ID_BE_BRU_WG_101),
            ),
        locations = listOf(RELAY_BE_BRU_WG_101),
    )
