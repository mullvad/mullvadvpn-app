package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class DaitaSettings(val enabled: Boolean) {
    companion object
}
