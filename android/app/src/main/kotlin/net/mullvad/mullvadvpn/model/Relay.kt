package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class Relay(
    val hostname: String,
    val active: Boolean,
    val tunnels: RelayTunnels
) : Parcelable {
    val hasWireguardTunnels
        get() = !tunnels.wireguard.isEmpty()
}
