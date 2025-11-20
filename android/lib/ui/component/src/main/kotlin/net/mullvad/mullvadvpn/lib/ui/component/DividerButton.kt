package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive

@Preview
@Composable
private fun PreviewDividerButton() {
    AppTheme { Box(modifier = Modifier.height(56.dp)) { DividerButton(icon = Icons.Default.Add) } }
}

@Composable
fun DividerButton(
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    icon: ImageVector,
    onClick: () -> Unit = {},
) {
    Row(modifier) {
        VerticalDivider(thickness = VerticalDividerWidth)
        Box(
            modifier =
                Modifier.width(DividerButtonWidth)
                    .fillMaxHeight()
                    .clickable(enabled = isEnabled, onClick = onClick),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint =
                    if (isEnabled) LocalContentColor.current
                    else LocalContentColor.current.copy(AlphaInactive),
            )
        }
    }
}

private val DividerButtonWidth = 64.dp
private val VerticalDividerWidth = 2.dp
