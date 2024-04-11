package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics
data class RelayConstraints(
    val location: Constraint<LocationConstraint>,
    val providers: Constraint<Providers>,
    val ownership: Constraint<Ownership>,
    val wireguardConstraints: WireguardConstraints,
) {
    companion object
}
