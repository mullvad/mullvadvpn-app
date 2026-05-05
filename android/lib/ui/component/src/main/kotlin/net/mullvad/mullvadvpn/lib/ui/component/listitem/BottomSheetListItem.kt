package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Composable
fun BottomSheetListItem(
    modifier: Modifier = Modifier,
    title: String,
    backgroundColor: Color,
    onBackgroundColor: Color,
    singleLine: Boolean = true,
    isEnabled: Boolean = true,
    onClick: (() -> Unit)? = null,
) {
    MullvadListItem(
        modifier = modifier,
        position = Position.Middle,
        content = { Text(text = title, maxLines = if (singleLine) 1 else Int.MAX_VALUE) },
        colors =
            ListItemDefaults.colors(
                headlineColor = onBackgroundColor,
                containerColorParent = backgroundColor,
            ),
        onClick = onClick,
        isEnabled = isEnabled,
    )
}
