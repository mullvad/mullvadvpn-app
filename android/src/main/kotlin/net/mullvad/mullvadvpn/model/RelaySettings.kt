package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelaySettings : Parcelable {
    @Parcelize
    @Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
    class CustomTunnelEndpoint() : RelaySettings(), Parcelable

    @Parcelize
    class Normal(var relayConstraints: RelayConstraints) : RelaySettings(), Parcelable
}
