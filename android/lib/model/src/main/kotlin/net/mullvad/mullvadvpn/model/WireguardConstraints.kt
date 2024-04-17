package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class WireguardConstraints(val port: Constraint<Port>) {
    companion object
}
