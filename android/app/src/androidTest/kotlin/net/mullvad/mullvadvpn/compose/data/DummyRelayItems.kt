package net.mullvad.mullvadvpn.compose.data

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.RelayEndpointData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.model.WireguardEndpointData
import net.mullvad.mullvadvpn.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.toRelayCountries

private val DUMMY_RELAY_1 =
    net.mullvad.mullvadvpn.model.Relay(
        hostname = "Relay host 1",
        active = true,
        endpointData = RelayEndpointData.Wireguard(WireguardRelayEndpointData),
        owned = true,
        provider = "PROVIDER"
    )
private val DUMMY_RELAY_2 =
    net.mullvad.mullvadvpn.model.Relay(
        hostname = "Relay host 2",
        active = true,
        endpointData = RelayEndpointData.Wireguard(WireguardRelayEndpointData),
        owned = true,
        provider = "PROVIDER"
    )
private val DUMMY_RELAY_CITY_1 = RelayListCity("Relay City 1", "RCi1", arrayListOf(DUMMY_RELAY_1))
private val DUMMY_RELAY_CITY_2 = RelayListCity("Relay City 2", "RCi2", arrayListOf(DUMMY_RELAY_2))
private val DUMMY_RELAY_COUNTRY_1 =
    RelayListCountry("Relay Country 1", "RCo1", arrayListOf(DUMMY_RELAY_CITY_1))
private val DUMMY_RELAY_COUNTRY_2 =
    RelayListCountry("Relay Country 2", "RCo2", arrayListOf(DUMMY_RELAY_CITY_2))

private val DUMMY_WIREGUARD_PORT_RANGES = ArrayList<PortRange>()
private val DUMMY_WIREGUARD_ENDPOINT_DATA = WireguardEndpointData(DUMMY_WIREGUARD_PORT_RANGES)

val DUMMY_RELAY_COUNTRIES =
    RelayList(
            arrayListOf(DUMMY_RELAY_COUNTRY_1, DUMMY_RELAY_COUNTRY_2),
            DUMMY_WIREGUARD_ENDPOINT_DATA,
        )
        .toRelayCountries(ownership = Constraint.Any(), providers = Constraint.Any())

val DUMMY_CUSTOM_LISTS =
    listOf(
        RelayItem.CustomList("First list", false, "1", locations = DUMMY_RELAY_COUNTRIES),
        RelayItem.CustomList("Empty list", expanded = false, "2", locations = emptyList())
    )
