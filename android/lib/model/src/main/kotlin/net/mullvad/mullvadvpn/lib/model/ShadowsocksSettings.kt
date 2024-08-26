package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class ShadowsocksSettings(val port: Constraint<Port>) {
    companion object
}
