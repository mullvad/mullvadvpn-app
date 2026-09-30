package net.mullvad.mullvadvpn.lib.common.compose

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.buildAnnotatedString

fun Iterable<AnnotatedString>.joinToAnnotatedString(
    separator: AnnotatedString = AnnotatedString(", ")
): AnnotatedString = buildAnnotatedString {
    val iterator = this@joinToAnnotatedString.iterator()
    while (iterator.hasNext()) {
        append(iterator.next())

        if (iterator.hasNext()) {
            append(separator)
        }
    }
}

fun Iterable<AnnotatedString>.joinToAnnotatedString(separator: String): AnnotatedString =
    joinToAnnotatedString(AnnotatedString(separator))
