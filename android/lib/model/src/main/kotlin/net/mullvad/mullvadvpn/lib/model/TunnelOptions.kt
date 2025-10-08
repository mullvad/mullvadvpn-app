package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class TunnelOptions(
    val mtu: Mtu?,
    val quantumResistant: QuantumResistantState,
    val daitaSettings: DaitaSettings,
    val dnsOptions: DnsOptions,
    val enableIpv6: Boolean,
) {
    companion object
}
