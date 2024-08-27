package net.mullvad.mullvadvpn.test.e2e.misc

import java.net.Inet4Address
import java.net.NetworkInterface
import junit.framework.TestCase.fail

class Networking {
    companion object {
        fun getIPAddress(): String {
            NetworkInterface.getNetworkInterfaces()?.toList()?.map { networkInterface ->
                networkInterface.inetAddresses
                    ?.toList()
                    ?.find { !it.isLoopbackAddress && it is Inet4Address }
                    ?.let {
                        return it.hostAddress
                    }
            }

            fail("Failed to get test device IP address")
            return ""
        }
    }
}
