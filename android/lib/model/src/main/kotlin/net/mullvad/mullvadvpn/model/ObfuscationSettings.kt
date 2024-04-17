package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class ObfuscationSettings(
    val selectedObfuscation: SelectedObfuscation,
    val udp2tcp: Udp2TcpObfuscationSettings
) {
    companion object
}
