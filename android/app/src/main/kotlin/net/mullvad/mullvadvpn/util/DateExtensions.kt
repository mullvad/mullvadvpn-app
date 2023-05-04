package net.mullvad.mullvadvpn.util

import org.joda.time.DateTime
import org.joda.time.format.ISODateTimeFormat

fun DateTime.formatDate(): String = ISODateTimeFormat.date().print(this)
