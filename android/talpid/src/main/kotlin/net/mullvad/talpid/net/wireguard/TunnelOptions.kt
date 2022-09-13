package net.mullvad.talpid.net.wireguard

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class TunnelOptions(
    val mtu: Int?,
    val usePqSafePsk: Boolean
) : Parcelable
