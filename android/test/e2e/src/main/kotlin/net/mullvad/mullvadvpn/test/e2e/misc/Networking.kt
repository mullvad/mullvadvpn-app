package net.mullvad.mullvadvpn.test.e2e.misc

import java.net.Inet4Address
import java.net.NetworkInterface
import org.junit.Assert.fail

object Networking {
    fun getDeviceIpv4Address(): String {
        NetworkInterface.getNetworkInterfaces()!!.toList().map { networkInterface ->
            val address =
                networkInterface.inetAddresses.toList().find {
                    !it.isLoopbackAddress && it is Inet4Address
                }

            if (address != null && address.hostAddress != null) {
                return address.hostAddress!!
            }
        }

        fail("Failed to get test device IP address")
        return ""
    }
}
