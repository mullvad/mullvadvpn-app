package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelayEndpointType : Parcelable {
    @Parcelize object Openvpn : RelayEndpointType()

    @Parcelize object Bridge : RelayEndpointType()

    @Parcelize
    data class Wireguard(val wireguardRelayEndpointData: WireguardRelayEndpointData) :
        RelayEndpointType()
}
