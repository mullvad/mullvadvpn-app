package net.mullvad.mullvadvpn.model

import net.mullvad.talpid.net.wireguard.TunnelOptions as TalpidWireguardTunnelOptions

data class WireguardTunnelOptions(val options: TalpidWireguardTunnelOptions)
