package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class Settings(
    val relaySettings: RelaySettings,
    val obfuscationSettings: ObfuscationSettings,
    val customLists: List<CustomList>,
    val allowLan: Boolean,
    val autoConnect: Boolean,
    val tunnelOptions: TunnelOptions,
    val relayOverrides: List<RelayOverride>,
    val showBetaReleases: Boolean,
    val splitTunnelSettings: SplitTunnelSettings
) {
    companion object
}
