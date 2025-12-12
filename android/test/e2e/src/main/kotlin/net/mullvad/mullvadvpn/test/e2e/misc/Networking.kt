package net.mullvad.mullvadvpn.test.e2e.misc

import java.net.Inet4Address
import java.net.Inet6Address
import java.net.NetworkInterface

data class IpAddrs(val ipv4: String, val ipv6: List<String>)

object Networking {
    fun getDeviceIpAddrs(): IpAddrs {
        NetworkInterface.getNetworkInterfaces()!!.toList().forEach { networkInterface ->
            val v4 =
                networkInterface.inetAddresses.toList().find {
                    !it.isLoopbackAddress && it is Inet4Address
                }

            val v6 =
                networkInterface.inetAddresses
                    .toList()
                    .filter { !it.isLoopbackAddress && it is Inet6Address }
                    .map { it.hostAddress!! }

            if (v4 != null && v4.hostAddress != null) {
                return IpAddrs(v4.hostAddress!!, v6)
            }
        }

        error("Failed to get test device IP address")
    }
}
