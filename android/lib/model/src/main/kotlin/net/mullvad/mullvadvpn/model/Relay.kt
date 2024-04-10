package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Relay(
    val hostname: String,
    val active: Boolean,
    val ownership: Ownership,
    val provider: ProviderId,
    val endpointType: RelayEndpointType
) : Parcelable {
    val isWireguardRelay
        get() = endpointType is RelayEndpointType.Wireguard
}
