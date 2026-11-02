package net.mullvad.mullvadvpn.lib.model

import java.net.InetAddress

data class TunnelConfig(val privateKey: WireguardKey, val tunnelIps: List<InetAddress>)
