package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class WireguardConstraints(
    val multihop: MultihopMode,
    val entryLocation: Constraint<RelayItemId>,
    val entryOwnership: Constraint<Ownership>,
    val entryProviders: Constraint<Providers>,
    val ipVersion: Constraint<IpVersion>,
) {
    companion object
}

enum class MultihopMode {
    WHEN_NEEDED,
    ALWAYS,
    NEVER,
}
