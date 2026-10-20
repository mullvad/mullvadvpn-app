package net.mullvad.talpid.model

import java.net.InetAddress
import kotlin.test.assertEquals
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory

class ConnectivityTest {
    @TestFactory
    fun test_isUsableForInternet() =
        listOf(
                // Unknown source IP
                "0.0.0.0" to false,
                // Public IP's
                "1.1.1.1" to true,
                "8.8.8.8" to true,
                // Localhost address
                "127.0.0.1" to false,
                // Localhost address
                "169.254.0.1" to false,
                // Normal LAN addresses (Most likely using NAT)
                "10.0.0.2" to true,
                "192.168.0.2" to true,
                // Multicast address
                "224.0.0.1" to false,
                // Global unique IP
                "2a03:1b20:bef0:100:7970:2962:d5f9:8c20" to true,
                // Link local address
                "fe80::73:1b59:5a06:9a09" to false,
                // Unique local address (Could be using NPTv6/NAT66)
                "fc00::1" to true,
                // Reserved
                "::" to false,
                // IPv6 Loopback address
                "::1" to false,
                // IPv4 mapped address
                "::ffff:1.1.1.1" to true,
            )
            .map { InetAddress.getByName(it.first) to it.second }
            .map { (input, expected) ->
                DynamicTest.dynamicTest(
                    "when address is ${input.hostAddress} it should return $expected"
                ) {
                    assertEquals(
                        expected,
                        input.isUsableForInternet(),
                        "Expected isUsableForInternet() for ${input.hostAddress} to be $expected",
                    )
                }
            }
}
