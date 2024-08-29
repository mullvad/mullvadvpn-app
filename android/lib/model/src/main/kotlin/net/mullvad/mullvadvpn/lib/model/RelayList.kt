package net.mullvad.mullvadvpn.lib.model

data class RelayList(
    val countries: List<RelayItem.Location.Country>,
    val wireguardEndpointData: WireguardEndpointData,
)
