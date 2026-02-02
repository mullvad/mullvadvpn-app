package net.mullvad.mullvadvpn.lib.ui.component.text

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

/** Text placed below list items to for example explain why it is disabled. */
@Composable
fun ListItemInfo(
    modifier: Modifier = Modifier,
    text: String,
    style: TextStyle = MaterialTheme.typography.bodyMedium,
    color: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    Text(
        text = text,
        style = style,
        color = color,
        modifier =
            modifier
                .padding(
                    top = Dimens.tinyPadding,
                    start = Dimens.mediumPadding,
                    end = Dimens.mediumPadding,
                    bottom = Dimens.tinyPadding,
                )
                .fillMaxWidth()
                .wrapContentHeight(),
    )
}
