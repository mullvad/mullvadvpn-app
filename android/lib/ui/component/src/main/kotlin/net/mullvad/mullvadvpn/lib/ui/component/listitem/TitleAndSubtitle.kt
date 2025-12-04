package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

@Composable
internal fun TitleAndSubtitle(
    title: String,
    subtitle: String?,
    subtitleColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    Column {
        Text(title)
        if (subtitle != null) {
            Text(
                text = subtitle,
                style = MaterialTheme.typography.labelLarge,
                color = subtitleColor,
            )
        }
    }
}
