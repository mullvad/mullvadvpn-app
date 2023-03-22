package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelayEndpointData : Parcelable {
    @Parcelize object Openvpn : RelayEndpointData()

    @Parcelize object Bridge : RelayEndpointData()

    @Parcelize
    data class Wireguard(val wireguardRelayEndpointData: WireguardRelayEndpointData) :
        RelayEndpointData()
}
