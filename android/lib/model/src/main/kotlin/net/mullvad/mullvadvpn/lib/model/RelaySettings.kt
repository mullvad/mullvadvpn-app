package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class RelaySettings(val relayConstraints: RelayConstraints) {
    companion object
}
