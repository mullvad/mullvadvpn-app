package net.mullvad.mullvadvpn.compose.extensions

import android.graphics.Typeface
import android.text.Spanned
import android.text.style.StyleSpan
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight

fun Spanned.toAnnotatedString(boldFontWeight: FontWeight = FontWeight.Bold): AnnotatedString =
    buildAnnotatedString {
        val spanned = this@toAnnotatedString
        append(spanned.toString())
        getSpans(0, spanned.length, Any::class.java).forEach { span ->
            val start = getSpanStart(span)
            val end = getSpanEnd(span)
            when (span) {
                is StyleSpan ->
                    when (span.style) {
                        Typeface.BOLD ->
                            addStyle(SpanStyle(fontWeight = boldFontWeight), start, end)
                    }
            }
        }
    }
