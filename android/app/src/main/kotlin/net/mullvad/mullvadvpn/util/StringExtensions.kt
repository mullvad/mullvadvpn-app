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

fun String.isValidMtu(): Boolean {
    return this.toIntOrNull()?.let {
        it in 1280..1420
    } ?: run { true }
}
