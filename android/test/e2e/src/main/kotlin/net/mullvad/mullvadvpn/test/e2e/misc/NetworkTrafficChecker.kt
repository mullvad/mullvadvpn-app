package net.mullvad.mullvadvpn.test.e2e.misc

import net.mullvad.mullvadvpn.test.e2e.router.packetCapture.Stream
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertTrue

object NetworkTrafficChecker {
    fun checkTrafficStreamsAgainstRules(streams: List<Stream>, vararg rules: TrafficRule) {
        // Assert that there are streams to be analyzed. Stream objects are guaranteed to contain
        // packets when initialized.
        assertTrue(streams.isNotEmpty(), "List of streams is empty.")

        for (rule in rules) {
            rule.assertTraffic(streams)
        }
    }
}

interface TrafficRule {
    fun assertTraffic(streams: List<Stream>)
}

class NoTrafficToHostRule(private val host: String) : TrafficRule {
    override fun assertTraffic(streams: List<Stream>) {
        streams.forEach { assertNotEquals(host, it.destinationHost.ipAddress) }
    }
}

class SomeTrafficToHostRule(private val host: String) : TrafficRule {
    override fun assertTraffic(streams: List<Stream>) {
        val hasAnyTrafficToSpecifiedHost = streams.any { it.destinationHost.ipAddress == host }
        assertTrue(
            hasAnyTrafficToSpecifiedHost,
            "Expected some traffic to the specified host ($host)," +
                "but all traffic had other destinations addresses.",
        )
    }
}

class SomeTrafficToOtherHostsRule(private val hostToExclude: String) : TrafficRule {
    override fun assertTraffic(streams: List<Stream>) {
        val hasAnyTrafficToOtherHost = streams.any { it.destinationHost.ipAddress != hostToExclude }
        assertTrue(
            hasAnyTrafficToOtherHost,
            "Expected some traffic to leak, but all traffic had destination address: $hostToExclude",
        )
    }
}
