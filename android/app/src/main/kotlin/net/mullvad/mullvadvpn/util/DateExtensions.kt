package net.mullvad.mullvadvpn.util

import java.text.DateFormat
import org.joda.time.DateTime
import org.joda.time.format.ISODateTimeFormat

fun DateTime.formatDate(): String = ISODateTimeFormat.date().print(this)

fun DateTime.toExpiryDateString(): String =
    DateFormat.getDateTimeInstance(DateFormat.MEDIUM, DateFormat.SHORT).format(this.toDate())
