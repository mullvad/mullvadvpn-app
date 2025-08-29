package net.mullvad.mullvadvpn.util

import android.text.Html
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.withStyle
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

/**
 * Appends `text` and styles occurrences of `substring` in `text` with the given `substringStyle`.
 */
fun AnnotatedString.Builder.appendTextWithStyledSubstring(
    text: String,
    substring: String,
    substringStyle: SpanStyle,
    ignoreCase: Boolean = false,
    limit: Int = 0,
) {
    val parts = text.split(substring, ignoreCase = ignoreCase, limit = limit)

    parts.forEachIndexed { index, part ->
        append(part)
        if (index != parts.lastIndex) {
            withStyle(substringStyle) { append(substring) }
        }
    }
}
