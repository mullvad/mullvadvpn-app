package net.mullvad.mullvadvpn.test.mockapi.util

import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

private const val STRICT_ISO8601_AND_RFC3339_PATTERN = "yyyy-MM-dd'T'HH:mm:ssZZ"

fun currentUtcTimeWithOffsetZero() = DateTime.now(DateTimeZone.forOffsetHours(0))

fun DateTime.formatStrictlyAccordingToIso8601AndRfc3339(): String {
    return toString(DateTimeFormat.forPattern(STRICT_ISO8601_AND_RFC3339_PATTERN))
}
