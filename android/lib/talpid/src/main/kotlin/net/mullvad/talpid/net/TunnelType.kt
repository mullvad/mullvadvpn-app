package net.mullvad.talpid.net

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class TunnelType : Parcelable {
    OpenVpn,
    Wireguard
}
