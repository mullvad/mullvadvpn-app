package net.mullvad.mullvadvpn.lib.ui.component.text

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.AnnotatedString

/** Text that is used at the top of the screen, it gives information about the screen. */
@Composable
fun Description(text: AnnotatedString, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier,
    )
}

@Composable
fun Description(text: String, modifier: Modifier = Modifier) {
    Description(text = AnnotatedString(text), modifier = modifier)
}
