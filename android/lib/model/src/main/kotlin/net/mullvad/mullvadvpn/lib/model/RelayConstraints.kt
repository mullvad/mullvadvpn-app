package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class RelayConstraints(
    val location: Constraint<net.mullvad.mullvadvpn.lib.model.RelayItemId>,
    val providers: Constraint<Providers>,
    val ownership: Constraint<Ownership>,
    val wireguardConstraints: WireguardConstraints,
) {
    companion object
}
