package net.mullvad.mullvadvpn.lib.model

data class WireguardEndpointData(
    val portRanges: List<PortRange>,
    val shadowsocksPortRanges: List<PortRange>
)
