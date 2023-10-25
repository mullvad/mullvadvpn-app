package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayConstraints(
    val location: Constraint<LocationConstraint>,
    val providers: Constraint<Providers>,
    val ownership: Constraint<Ownership>,
    val wireguardConstraints: WireguardConstraints,
) : Parcelable
