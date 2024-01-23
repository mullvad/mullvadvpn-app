package net.mullvad.mullvadvpn.lib.common.util

import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

private const val EXPIRY_FORMAT = "YYYY-MM-dd HH:mm:ss z"
private const val BIG_DOT_CHAR = "●"
private const val SPACE_CHAR = ' '

fun String.parseAsDateTime(): DateTime? {
    return try {
        DateTime.parse(this, DateTimeFormat.forPattern(EXPIRY_FORMAT))
    } catch (ex: IllegalArgumentException) {
        null
    } catch (ex: UnsupportedOperationException) {
        null
    }
}

fun String.groupWithSpaces(groupCharSize: Int = 4): String {
    return fold(StringBuilder()) { formattedText, nextDigit ->
            if ((formattedText.length % (groupCharSize + 1)) == groupCharSize) {
                formattedText.append(SPACE_CHAR)
            }
            formattedText.append(nextDigit)
        }
        .toString()
}

fun String.groupPasswordModeWithSpaces(groupCharSize: Int = 4): String {
    return BIG_DOT_CHAR.repeat(this.length).groupWithSpaces(groupCharSize)
}
