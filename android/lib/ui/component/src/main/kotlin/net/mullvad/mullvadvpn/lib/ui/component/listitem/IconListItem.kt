package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountBalance
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewExternalLinkListItem() {
    AppTheme {
        PreviewSpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            IconListItem(title = "Navigation sample", leadingIcon = Icons.Default.Settings, onClick = {})
            IconListItem(title = "Navigation sample", leadingIcon = Icons.Default.AccountBalance, onClick = {})
        }
    }
}

@Composable
fun IconListItem(
    modifier: Modifier = Modifier,
    title: String,
    leadingIcon: ImageVector,
    colors: ListItemColors = ListItemDefaults.colors(),
    contentDescription: String? = null,
    position: Position = Position.Single,
    isEnabled: Boolean = true,
    onClick: () -> Unit,
) {
    MullvadListItem(
        modifier = modifier,
        content = { Text(title) },
        leadingContent = {
            Icon(
                imageVector = leadingIcon,
                modifier = Modifier.padding(end = Dimens.smallPadding),
                contentDescription = contentDescription,
            )
        },
        position = position,
        colors = colors,
        onClick = onClick,
        isEnabled = isEnabled,
    )
}
