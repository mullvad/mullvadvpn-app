package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class RelayConstraints(
    val location: Constraint<LocationConstraint>,
    val providers: Constraint<Providers>,
    val ownership: Constraint<Ownership>,
    val wireguardConstraints: WireguardConstraints,
) {
    companion object
}
