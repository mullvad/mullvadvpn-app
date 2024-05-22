package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class Providers(val providers: Set<ProviderId>) {
    companion object
}
