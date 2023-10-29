package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class RelaySettings : Parcelable {
    @Parcelize data object CustomTunnelEndpoint : RelaySettings()

    @Parcelize data class Normal(val relayConstraints: RelayConstraints) : RelaySettings()

    fun relayConstraints(): RelayConstraints? = (this as? Normal)?.relayConstraints
}
