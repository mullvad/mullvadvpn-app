package net.mullvad.mullvadvpn.lib.model

data class CustomVpnConfig(
    val tunnelConfig: TunnelConfig,
    val peerConfig: List<PeerConfig>,
    val enabled: Boolean,
    val name: String?,
)
