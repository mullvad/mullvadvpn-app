package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelayEndpointType : Parcelable {
    @Parcelize data object Openvpn : RelayEndpointType()

    @Parcelize data object Bridge : RelayEndpointType()

    @Parcelize data object Wireguard : RelayEndpointType()
}
