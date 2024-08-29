package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics
import java.net.InetAddress

@optics
data class RelayOverride(
    val hostname: String,
    val ipv4AddressIn: InetAddress?,
    val ipv6AddressIn: InetAddress?,
) {
    companion object
}
