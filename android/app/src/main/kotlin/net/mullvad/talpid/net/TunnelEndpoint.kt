package net.mullvad.talpid.net

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class TunnelEndpoint(val endpoint: Endpoint, val quantumResistant: Boolean) : Parcelable
