package net.mullvad.mullvadvpn.util

import java.text.DateFormat
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.DurationUnit
import org.joda.time.DateTime
import org.joda.time.format.ISODateTimeFormat

fun DateTime.formatDate(): String = ISODateTimeFormat.date().print(this)

fun DateTime.toExpiryDateString(): String =
    DateFormat.getDateTimeInstance(DateFormat.MEDIUM, DateFormat.SHORT).format(this.toDate())

fun DateTime.daysFromNow() =
    (toInstant().millis - DateTime.now().toInstant().millis).milliseconds.toInt(DurationUnit.DAYS)
