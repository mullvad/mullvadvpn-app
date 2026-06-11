package net.mullvad.mullvadvpn.lib.common.compose

import androidx.annotation.StringRes
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.unit.em

data class IconString(val text: AnnotatedString, val inlineContent: Map<String, InlineTextContent>)

data class DescribedIcon(val icon: ImageVector, val contentDescription: String)

/**
 * Given a string res with any number of string format arguments (e.g. %1$s), returns an annotated
 * string together with inline content where each format arg will be substituted with the
 * corresponding icon from `icons`. So %1$s is substituted with icons[0], %2$s with icons[1] and so
 * on.
 */
@Composable
fun stringResourceWithIcons(@StringRes id: Int, vararg icons: DescribedIcon): IconString {
    require(icons.isNotEmpty()) { "icons cannot be empty" }

    val iconIds = icons.mapIndexed { index, _ -> "[[icon_id_${index + 1}]]" }

    // Replace all args in the string with the corresponding icon id.
    val text = stringResource(id, *iconIds.toTypedArray())

    // Find all places in the text where we have an inline icon.
    val idRegex = Regex(iconIds.joinToString("|") { Regex.escape(it) })
    val matches = idRegex.findAll(text)

    val annotatedString = buildAnnotatedString {
        var lastIndex = 0

        matches.forEach { match ->
            val iconId = match.value
            val icon = icons[iconIds.indexOf(iconId)]

            val beforeIcon = text.substring(lastIndex, match.range.first)

            append(beforeIcon)

            appendInlineContent(id = iconId, alternateText = icon.contentDescription)

            lastIndex = match.range.last + 1
        }

        // Append the rest of the text that is after the last icon.
        append(text.substring(lastIndex))
    }

    val inlineContent = iconIds.associateWith { iconId ->
        val icon = icons[iconIds.indexOf(iconId)]

        InlineTextContent(
            Placeholder(
                width = 1.5.em,
                height = 1.5.em,
                placeholderVerticalAlign = PlaceholderVerticalAlign.TextCenter,
            )
        ) {
            Icon(imageVector = icon.icon, contentDescription = icon.contentDescription)
        }
    }

    return IconString(text = annotatedString, inlineContent = inlineContent)
}
