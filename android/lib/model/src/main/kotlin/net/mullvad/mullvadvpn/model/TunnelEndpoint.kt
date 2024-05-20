package net.mullvad.mullvadvpn.model

data class TunnelEndpoint(
    val endpoint: Endpoint,
    val quantumResistant: Boolean,
    val obfuscation: ObfuscationEndpoint?
)
