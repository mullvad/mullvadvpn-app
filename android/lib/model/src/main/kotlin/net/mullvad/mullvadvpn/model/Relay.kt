package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Relay(val hostname: String, val active: Boolean, val endpointData: RelayEndpointData) :
    Parcelable {
    val isWireguardRelay
        get() = endpointData is RelayEndpointData.Wireguard
}
