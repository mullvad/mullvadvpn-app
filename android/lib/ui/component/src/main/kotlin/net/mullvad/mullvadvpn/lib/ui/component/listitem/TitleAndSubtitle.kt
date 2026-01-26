package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextDirection

@Composable
internal fun TitleAndSubtitle(
    title: String,
    subtitle: String?,
    subtitleColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    subTitleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    subTitleTextDirection: TextDirection = TextDirection.Unspecified,
) {
    Column {
        Text(title)
        if (subtitle != null) {
            Text(
                text = subtitle,
                style = subTitleStyle.copy(textDirection = subTitleTextDirection),
                color = subtitleColor,
            )
        }
    }
}
