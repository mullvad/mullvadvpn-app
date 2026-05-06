package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.text.style.TextOverflow
import kotlin.Int

@Composable
internal fun TitleAndSubtitle(
    title: String,
    subtitle: String?,
    singleLine: Boolean,
    subtitleColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    subTitleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    subTitleTextDirection: TextDirection = TextDirection.Unspecified,
) {
    Column {
        Text(
            text = title,
            maxLines = if (singleLine) 1 else Int.MAX_VALUE,
            overflow = TextOverflow.Ellipsis,
        )
        if (subtitle != null) {
            Text(
                text = subtitle,
                style = subTitleStyle.copy(textDirection = subTitleTextDirection),
                color = subtitleColor,
                maxLines = if (singleLine) 1 else Int.MAX_VALUE,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
