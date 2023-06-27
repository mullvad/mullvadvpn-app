package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayList(
    val countries: ArrayList<RelayListCountry>,
    val wireguardEndpointData: WireguardEndpointData
) : Parcelable
