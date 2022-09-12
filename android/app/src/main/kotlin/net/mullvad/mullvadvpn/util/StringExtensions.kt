package net.mullvad.mullvadvpn.util

import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

private const val EXPIRY_FORMAT = "YYYY-MM-dd HH:mm:ss z"

fun String.capitalizeFirstCharOfEachWord(): String {
    return split(" ")
        .joinToString(" ") { word ->
            word.replaceFirstChar { firstChar -> firstChar.uppercase() }
        }
        .trimEnd()
}

fun String.parseAsDateTime(): DateTime? {
    return try {
        DateTime.parse(this, DateTimeFormat.forPattern(EXPIRY_FORMAT))
    } catch (ex: Exception) {
        null
    }
}
