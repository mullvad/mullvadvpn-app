package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.LinkInteractionListener
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle

/**
 * Creates an [AnnotatedString] from a localized string with a clickable part. The [text] parameter
 * should contain a single "%s" placeholder where the [argument] will be inserted.
 */
fun clickableAnnotatedString(
    text: String,
    argument: String,
    linkStyle: SpanStyle,
    onClick: (String) -> Unit,
) = buildAnnotatedString {
    val firstString = text.substringBefore("%s")
    val secondString = text.substringAfter("%s")
    append(firstString)
    withLink(
        link =
            LinkAnnotation.Clickable(
                tag = argument,
                linkInteractionListener = LinkInteractionListener { onClick(argument) },
            ),
        block = { withStyle(style = linkStyle) { append(argument) } },
    )
    append(secondString)
}
