package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class WireguardConstraints(
    val port: Constraint<Port>,
    val isMultihopEnabled: Boolean,
    val entryLocation: Constraint<RelayItemId>,
    val ipVersion: Constraint<IpVersion>,
) {
    companion object
}
