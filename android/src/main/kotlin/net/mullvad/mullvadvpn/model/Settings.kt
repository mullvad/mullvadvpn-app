package net.mullvad.mullvadvpn.model

data class Settings(
    val accountToken: String?,
    val relaySettings: RelaySettings,
    val allowLan: Boolean,
    val autoConnect: Boolean,
    val tunnelOptions: TunnelOptions,
    val showBetaReleases: Boolean
)
