package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class ObfuscationSettings(
    val selectedObfuscationMode: ObfuscationMode,
    val udp2tcp: Udp2TcpObfuscationSettings,
    val shadowsocks: ShadowsocksSettings,
    val wireguardPort: Constraint<Port>,
) {
    companion object
}
