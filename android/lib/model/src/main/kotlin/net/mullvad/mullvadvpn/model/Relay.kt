package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Relay(
    val hostname: String,
    val active: Boolean,
    val owned: Boolean,
    val provider: String,
    val endpointType: RelayEndpointType
) : Parcelable {
    val isWireguardRelay
        get() = endpointType is RelayEndpointType.Wireguard
}
