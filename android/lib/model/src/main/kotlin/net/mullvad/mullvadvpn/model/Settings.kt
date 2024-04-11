package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

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
) {
    companion object
}
