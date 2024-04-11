package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics
data class ObfuscationSettings(
    val selectedObfuscation: SelectedObfuscation,
    val udp2tcp: Udp2TcpObfuscationSettings
) {
    companion object
}
