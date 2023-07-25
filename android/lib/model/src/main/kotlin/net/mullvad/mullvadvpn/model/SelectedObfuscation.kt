package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class SelectedObfuscation : Parcelable {
    Auto,
    Off,
    Udp2Tcp
}
