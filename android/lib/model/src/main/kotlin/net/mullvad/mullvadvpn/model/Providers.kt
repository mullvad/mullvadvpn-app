package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class Providers(val providers: Set<ProviderId>) {
    companion object
}
