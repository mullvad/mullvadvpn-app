package net.mullvad.mullvadvpn.lib.model

import java.net.InetAddress

data class Quic(val inAddresses: List<InetAddress>) {
    val supportsIpv4 = inAddresses.any { it is java.net.Inet4Address }
    val supportsIpv6 = inAddresses.any { it is java.net.Inet6Address }
}
