package net.mullvad.talpid.tun_provider

import java.net.InetAddress
import java.net.Inet6Address

data class InetNetwork(val address: InetAddress, val prefixLength: Short) {
    val isIpv6 = address is Inet6Address
}
