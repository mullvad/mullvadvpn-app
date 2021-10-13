package net.mullvad.talpid.tun_provider

import java.net.Inet6Address
import java.net.InetAddress

data class InetNetwork(val address: InetAddress, val prefixLength: Short) {
    val isIpv6 = address is Inet6Address
}
