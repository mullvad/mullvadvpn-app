package net.mullvad.mullvadvpn.model

data class RelayList(
    val countries: List<RelayItem.Location.Country>,
    val wireguardEndpointData: WireguardEndpointData
)
