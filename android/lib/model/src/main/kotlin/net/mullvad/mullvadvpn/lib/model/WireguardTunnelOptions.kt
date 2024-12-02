package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class WireguardTunnelOptions(
    val mtu: Mtu?,
    val quantumResistant: QuantumResistantState,
    val daitaSettings: DaitaSettings,
) {
    companion object
}
