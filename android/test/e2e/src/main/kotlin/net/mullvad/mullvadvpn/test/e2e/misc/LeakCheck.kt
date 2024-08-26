package net.mullvad.mullvadvpn.test.e2e.misc

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue

object LeakCheck {
    fun assertNoLeaks(streams: List<Stream>, rules: List<LeakRule>) {
        // Assert that there are streams to be analyzed
        assertTrue(streams.isNotEmpty())

        for (rule in rules) {
            assertFalse(rule.isViolated(streams))
        }
    }

    fun assertLeaks(streams: List<Stream>, rules: List<LeakRule>) {
        for (rule in rules) {
            assertTrue(rule.isViolated(streams))
        }
    }
}

interface LeakRule {
    fun isViolated(streams: List<Stream>): Boolean
}

class NoTrafficToHostRule(private val host: String) : LeakRule {
    override fun isViolated(streams: List<Stream>): Boolean {
        return streams.any { it.destinationHost.ipAddress == host }
    }
}
