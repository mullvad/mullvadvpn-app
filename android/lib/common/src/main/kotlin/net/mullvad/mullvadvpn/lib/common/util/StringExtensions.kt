package net.mullvad.mullvadvpn.lib.common.util

import android.text.Html
import androidx.core.text.HtmlCompat

fun String.appendHideNavOnPlayBuild(isPlayBuild: Boolean): String =
    if (isPlayBuild) {
        "$this?hide_nav"
    } else {
        this
    }

fun String.removeHtmlTags(): String =
    Html.fromHtml(this, HtmlCompat.FROM_HTML_MODE_LEGACY).toString()

fun List<String>.trimAll() = map { it.trim() }

fun String.splitIncludingDelimiters(
    substring: String,
    ignoreCase: Boolean,
    limit: Int = 0,
): List<String> {
    if (substring.isEmpty()) return listOf(this)
    val result = mutableListOf<String>()
    var remaining = this
    while (remaining.isNotEmpty()) {
        val matchIndex = remaining.indexOf(substring, ignoreCase = ignoreCase)
        if (matchIndex == -1 || (limit > 0 && result.size >= limit * 2 - 1)) {
            result.add(remaining)
            break
        }
        if (matchIndex > 0) {
            result.add(remaining.substring(0, matchIndex))
        }
        result.add(remaining.substring(matchIndex, matchIndex + substring.length))
        remaining = remaining.substring(matchIndex + substring.length)
    }
    return result
}
