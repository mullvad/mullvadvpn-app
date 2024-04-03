package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class WireguardEndpointData(val portRanges: List<PortRange>) : Parcelable
