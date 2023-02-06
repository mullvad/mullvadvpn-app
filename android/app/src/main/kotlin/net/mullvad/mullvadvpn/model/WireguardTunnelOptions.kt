package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class WireguardTunnelOptions(
    val mtu: Int?,
    val quantumResistant: Boolean?
) : Parcelable
