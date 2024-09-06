package net.mullvad.mullvadvpn.lib.model

data class TunnelEndpoint(
    val endpoint: Endpoint,
    val quantumResistant: Boolean,
    val obfuscation: ObfuscationEndpoint?,
    val daita: Boolean,
)
