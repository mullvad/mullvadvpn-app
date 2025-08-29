package net.mullvad.mullvadvpn.lib.common.util

import co.touchlab.kermit.Logger
import java.time.DateTimeException
import java.time.Duration
import java.time.Instant
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

fun ZonedDateTime.formatDate(): String = DateTimeFormatter.ISO_LOCAL_DATE.format(this)

fun ZonedDateTime.toExpiryDateString(): String =
    try {
        DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).format(this)
    } catch (e: DateTimeException) {
        // This should normally not happen, but we have seen some crashes in the play console
        // where this exception is thrown, so fall back to ISO_LOCAL_DATE_TIME.
        // See: droid-2142
        Logger.e("Error formatting date with default locale: $e")
        DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(this)
    }

fun ZonedDateTime.millisFromNow(): Long = Duration.between(ZonedDateTime.now(), this).toMillis()

fun ZonedDateTime.daysFromNow(): Long = Duration.between(ZonedDateTime.now(), this).toDays()

fun ZonedDateTime.isBeforeNowInstant(): Boolean = toInstant().isBefore(Instant.now())

fun ZonedDateTime.isAfterNowInstant(): Boolean = toInstant().isAfter(Instant.now())
