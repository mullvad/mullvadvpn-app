package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class WireguardConstraints(val port: Constraint<Port>) {
    companion object
}
