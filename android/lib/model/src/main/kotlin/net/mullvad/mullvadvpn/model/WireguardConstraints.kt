package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics data class WireguardConstraints(val port: Constraint<Port>) {
    companion object
}
