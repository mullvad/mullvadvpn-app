package net.mullvad.talpid.model

import java.net.Inet6Address
import java.net.InetAddress

data class TunConfig(
    val addresses: ArrayList<InetAddress>,
    val dnsServers: ArrayList<InetAddress>,
    val routes: ArrayList<InetNetwork>,
    val excludedPackages: ArrayList<String>,
    val mtu: Int,
) {
    val hasIpv6Address: Boolean
        get() = addresses.any { it is Inet6Address }

    val hasIpv6DnsServer: Boolean
        get() = dnsServers.any { it is Inet6Address }

    val hasIpv6Route: Boolean
        get() = routes.any { it.address is Inet6Address }
}
