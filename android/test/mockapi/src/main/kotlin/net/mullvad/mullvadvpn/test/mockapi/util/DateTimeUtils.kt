package net.mullvad.mullvadvpn.test.mockapi.util

import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

private const val STRICT_ISO8601_AND_RFC3339_PATTERN = "yyyy-MM-dd'T'HH:mm:ssZZ"

fun ZonedDateTime.formatStrictlyAccordingToIso8601AndRfc3339(): String {
    return DateTimeFormatter.ofPattern(STRICT_ISO8601_AND_RFC3339_PATTERN).format(this)
}

fun ZonedDateTime.toExpiryDateString(): String =
    DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).format(this)
