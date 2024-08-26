package net.mullvad.mullvadvpn.test.e2e.misc

import java.net.Inet4Address
import java.net.NetworkInterface
import org.junit.Assert.fail

class Networking {
    companion object {
        fun getIpAddress(): String {
            NetworkInterface.getNetworkInterfaces()?.toList()?.map { networkInterface ->
                networkInterface.inetAddresses
                    ?.toList()
                    ?.find { !it.isLoopbackAddress && it is Inet4Address }
                    ?.let {
                        it.hostAddress?.let {
                            return it
                        }
                    }
            }

            fail("Failed to get test device IP address")
            return ""
        }
    }
}
