package net.mullvad.mullvadvpn.lib.ui.component.text

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview(name = "100%", fontScale = 1.0f)
@Preview(name = "130%", fontScale = 1.3f)
@Preview(name = "150%", fontScale = 1.5f)
@Composable
private fun PreviewFirstBaselineAlignedIconAndText() {
    AppTheme {
        Surface {
            FirstBaselineAlignedIconAndText(
                modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.mediumSpacer),
                text = "Filters are overridden when using an automatic location",
                icon = Icons.Rounded.Info,
                iconSize = Dimens.smallIconSize,
                iconTint = MaterialTheme.colorScheme.onSurface,
                textColor = MaterialTheme.colorScheme.onSurfaceVariant,
                textStyle = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
fun FirstBaselineAlignedIconAndText(
    modifier: Modifier = Modifier,
    text: String,
    icon: ImageVector,
    iconSize: Dp,
    iconTint: Color = MaterialTheme.colorScheme.onSurface,
    textColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    textStyle: TextStyle = MaterialTheme.typography.bodyMedium,
) {
    val textStyle = MaterialTheme.typography.bodyMedium

    val density = LocalDensity.current

    // Calculate vertical offset using the exact scaled font size.
    val centerToBaselineOffsetPx =
        with(density) { (textStyle.fontSize.toPx() * 0.35f).roundToInt() }

    val scalableIconSize = with(density) { iconSize.value.sp.toDp() }

    Row(modifier = modifier) {
        Icon(
            modifier =
                Modifier.size(scalableIconSize).alignBy { measurable ->
                    val iconCenter = measurable.measuredHeight / 2
                    iconCenter + centerToBaselineOffsetPx
                },
            imageVector = icon,
            tint = iconTint,
            contentDescription = null,
        )
        Spacer(modifier = Modifier.width(Dimens.tinyPadding))
        Text(
            modifier = Modifier.alignByBaseline(),
            style = textStyle,
            color = textColor,
            text = text,
        )
    }
}
