package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class RelaySettings(val relayConstraints: RelayConstraints) {
    companion object
}
