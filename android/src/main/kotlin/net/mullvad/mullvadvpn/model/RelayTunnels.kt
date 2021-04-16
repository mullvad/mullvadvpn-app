package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import java.util.ArrayList
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayTunnels(val wireguard: ArrayList<WireguardEndpointData>) : Parcelable
