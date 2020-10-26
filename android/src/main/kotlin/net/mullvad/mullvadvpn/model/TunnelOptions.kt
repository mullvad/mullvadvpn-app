package net.mullvad.mullvadvpn.model

import net.mullvad.talpid.net.wireguard.TunnelOptions as WireguardTunnelOptions

data class TunnelOptions(val wireguard: WireguardTunnelOptions, val dnsOptions: DnsOptions)
