package net.mullvad.mullvadvpn.util

import android.graphics.Typeface
import android.text.Html
import android.text.Spanned
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
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

fun CharSequence.toAnnotatedString(): AnnotatedString =
    if (this is Spanned) {
        buildAnnotatedString {
            append(this@toAnnotatedString.toString())
            val spans = getSpans(0, length, Object::class.java)

            spans.forEach { span ->
                val start = getSpanStart(span)
                val end = getSpanEnd(span)

                when (span) {
                    is StyleSpan ->
                        when (span.style) {
                            Typeface.BOLD ->
                                addStyle(SpanStyle(fontWeight = FontWeight.Bold), start, end)
                            Typeface.ITALIC ->
                                addStyle(SpanStyle(fontStyle = FontStyle.Italic), start, end)
                            Typeface.BOLD_ITALIC ->
                                addStyle(
                                    SpanStyle(
                                        fontWeight = FontWeight.Bold,
                                        fontStyle = FontStyle.Italic,
                                    ),
                                    start,
                                    end,
                                )
                        }
                    is UnderlineSpan ->
                        addStyle(SpanStyle(textDecoration = TextDecoration.Underline), start, end)
                }
            }
        }
    } else {
        AnnotatedString(this.toString())
    }
