package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelaySettings : Parcelable {
    @Parcelize
    object CustomTunnelEndpoint : RelaySettings(), Parcelable

    @Parcelize
    class Normal(val relayConstraints: RelayConstraints) : RelaySettings(), Parcelable
}
