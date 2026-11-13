package net.mullvad.mullvadvpn.lib.model

data class TunnelConfig(val privateKey: WireguardKey, val tunnelIps: Set<String>)
