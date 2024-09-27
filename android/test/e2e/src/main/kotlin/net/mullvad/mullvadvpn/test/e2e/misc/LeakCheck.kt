package net.mullvad.mullvadvpn.test.e2e.misc

import org.joda.time.DateTime
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue

object LeakCheck {
    fun assertNoLeaks(
        streams: List<Stream>,
        rules: List<LeakRule>,
        start: DateTime,
        end: DateTime,
    ) {
        for (rule in rules) {
            assertFalse(rule.isViolated(streams, start, end))
        }
    }

    fun assertLeaks(
        streams: List<Stream>,
        rules: List<LeakRule>,
        start: DateTime,
        end: DateTime,
    ) {
        for (rule in rules) {
            assertTrue(rule.isViolated(streams, start, end))
        }
    }
}

interface LeakRule {
    fun isViolated(streams: List<Stream>, start: DateTime, end: DateTime): Boolean
}

class NoTrafficToHostRule(private val host: String) : LeakRule {
    override fun isViolated(streams: List<Stream>, start: DateTime, end: DateTime): Boolean {
        return streams
            .filter { it.startDate.isAfter(start) && it.endDate.isBefore(end) }
            .any { it.destinationHost.ipAddress == host }
    }
}
