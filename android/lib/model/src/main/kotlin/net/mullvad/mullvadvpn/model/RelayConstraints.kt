package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.TunnelType

@Parcelize
data class RelayConstraints(
    val location: Constraint<LocationConstraint>,
    val providers: Constraint<Providers>,
    val ownership: Constraint<Ownership>,
    val tunnelProtocol: Constraint<TunnelType>,
    val wireguardConstraints: WireguardConstraints,
) : Parcelable
