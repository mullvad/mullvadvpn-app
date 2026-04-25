package net.mullvad.mullvadvpn.lib.ui.component

import android.graphics.Typeface
import android.text.Spanned
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import net.mullvad.mullvadvpn.lib.model.HighlightedString

fun HighlightedString.toAnnotatedString(highlightColor: Color): AnnotatedString =
    buildAnnotatedString {
        append(text)
        highlights.forEach {
            addStyle(SpanStyle(background = highlightColor), it.first, it.last + 1)
        }
    }

fun CharSequence.toAnnotatedString(): AnnotatedString =
    if (this is Spanned) {
        buildAnnotatedString {
            append(this@toAnnotatedString.toString())
            val spans = getSpans(0, length, Any::class.java)

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
