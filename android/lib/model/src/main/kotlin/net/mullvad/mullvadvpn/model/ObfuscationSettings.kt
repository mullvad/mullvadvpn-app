package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class ObfuscationSettings(
    val selectedObfuscation: SelectedObfuscation,
    val udp2tcp: Udp2TcpObfuscationSettings
) : Parcelable
