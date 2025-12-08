package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewSelectableListItem() {
    AppTheme {
        PreviewSpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            AppIconAndTitleListItem(
                appTitle = "Obfuscation",
                appIcon = R.drawable.ic_launcher_game_preview,
                isSelected = true,
                onClick = {},
            )
            AppIconAndTitleListItem(
                appTitle = "Obfuscation",
                appIcon = R.drawable.ic_launcher_game_preview,
                isSelected = true,
                onClick = {},
            )
        }
    }
}

/*
SelectableListItem.kt
 */

@Composable
fun AppIconAndTitleListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    appTitle: String,
    appIcon: Int,
    isSelected: Boolean,
    isEnabled: Boolean = true,
    appIconContentDescription: String? = null,
    onClick: (() -> Unit)? = null,
    testTag: String? = null,
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        isSelected = isSelected,
        testTag = testTag,
        onClick = onClick,
        content = {
            Row {
                Icon(
                    painter = painterResource(id = appIcon),
                    contentDescription = appIconContentDescription,
                    modifier = Modifier.size(APP_ICON_SIZE),
                    tint = Color.Unspecified,
                )
                Spacer(modifier = Modifier.width(Dimens.mediumPadding))
                Text(appTitle)
            }
        },
    )
}

private val APP_ICON_SIZE = 24.dp
