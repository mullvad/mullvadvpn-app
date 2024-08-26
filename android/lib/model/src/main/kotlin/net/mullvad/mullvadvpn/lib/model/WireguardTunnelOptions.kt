package net.mullvad.mullvadvpn.lib.model

data class WireguardTunnelOptions(
    val mtu: Mtu?,
    val quantumResistant: QuantumResistantState,
    val daita: Boolean,
)
