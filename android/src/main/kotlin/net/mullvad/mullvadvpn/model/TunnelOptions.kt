package net.mullvad.mullvadvpn.model

data class TunnelOptions(val wireguard: WireguardTunnelOptions, val dnsOptions: DnsOptions)
