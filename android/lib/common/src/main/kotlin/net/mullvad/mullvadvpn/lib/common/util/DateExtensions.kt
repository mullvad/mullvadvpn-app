package net.mullvad.mullvadvpn.lib.common.util

import java.time.Duration
import java.time.Instant
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

fun ZonedDateTime.formatDate(): String = DateTimeFormatter.ISO_LOCAL_DATE.format(this)

fun ZonedDateTime.toExpiryDateString(): String =
    DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).format(this)

fun ZonedDateTime.millisFromNow(): Long = Duration.between(ZonedDateTime.now(), this).toMillis()

fun ZonedDateTime.daysFromNow(): Long = Duration.between(ZonedDateTime.now(), this).toDays()

fun ZonedDateTime.isBeforeNowInstant(): Boolean = toInstant().isBefore(Instant.now())

fun ZonedDateTime.isAfterNowInstant(): Boolean = toInstant().isAfter(Instant.now())
