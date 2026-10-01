package net.mullvad.mullvadvpn.lib.ui.component.text

import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.AnnotatedString

/** Text that is used at the top of the screen, it gives information about the screen. */
@Composable
fun ScreenDescription(
    text: AnnotatedString,
    modifier: Modifier = Modifier,
    inlineContent: Map<String, InlineTextContent> = mapOf(),
) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        inlineContent = inlineContent,
        modifier = modifier,
    )
}

@Composable
fun ScreenDescription(
    text: String,
    modifier: Modifier = Modifier,
    inlineContent: Map<String, InlineTextContent> = mapOf(),
) {
    ScreenDescription(
        text = AnnotatedString(text),
        modifier = modifier,
        inlineContent = inlineContent,
    )
}
