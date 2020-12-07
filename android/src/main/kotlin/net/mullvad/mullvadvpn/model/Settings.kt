package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Settings(
    var accountToken: String?,
    var relaySettings: RelaySettings,
    var allowLan: Boolean,
    var autoConnect: Boolean,
    var tunnelOptions: TunnelOptions,
    var showBetaReleases: Boolean
) : Parcelable
