package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Settings(
    val accountToken: String?,
    val relaySettings: RelaySettings,
    val allowLan: Boolean,
    val autoConnect: Boolean,
    val tunnelOptions: TunnelOptions,
    val showBetaReleases: Boolean
) : Parcelable
