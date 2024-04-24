package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class TunnelEndpoint(
    val endpoint: Endpoint,
    val quantumResistant: Boolean,
    val obfuscation: ObfuscationEndpoint?
) : Parcelable
