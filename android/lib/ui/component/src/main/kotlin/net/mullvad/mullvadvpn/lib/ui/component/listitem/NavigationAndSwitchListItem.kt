package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSwitch
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewNavigationAndSwitchListItem() {
    AppTheme {
        NavigationAndSwitchListItem(
            title = "Title",
            subtitle = "Subtitle",
            isEnabled = true,
            isToggled = true,
            onCellClicked = {},
            onNavigationClicked = {},
        )
    }
}

@Composable
fun NavigationAndSwitchListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    subtitle: String?,
    isToggled: Boolean,
    isEnabled: Boolean = true,
    singeLine: Boolean = true,
    testTag: String? = null,
    backgroundAlpha: Float = 1f,
    onCellClicked: (Boolean) -> Unit,
    onNavigationClicked: (() -> Unit),
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        testTag = testTag,
        backgroundAlpha = backgroundAlpha,
        onClick = { onCellClicked(!isToggled) },
        content = {
            Column {
                Text(
                    text = title,
                    maxLines = if (singeLine) 1 else Int.MAX_VALUE,
                    overflow = TextOverflow.Ellipsis,
                )
                if (subtitle != null) {
                    Text(
                        text = subtitle,
                        maxLines = if (singeLine) 1 else Int.MAX_VALUE,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.labelLarge,
                        color =
                            if (isEnabled) MaterialTheme.colorScheme.onSurfaceVariant
                            else ListItemDefaults.colors().disabledHeadlineColor,
                    )
                }
            }
        },
        trailingContent = {
            Row(
                modifier = Modifier.fillMaxHeight(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(modifier = Modifier.fillMaxHeight().padding(end = Dimens.smallPadding)) {
                    MullvadSwitch(
                        modifier = Modifier.align(Alignment.Center),
                        checked = isToggled,
                        onCheckedChange = onCellClicked,
                        enabled = isEnabled,
                    )
                }
                Icon(
                    modifier = Modifier.clickable(enabled = isEnabled) { onNavigationClicked() },
                    imageVector = Icons.AutoMirrored.Rounded.KeyboardArrowRight,
                    contentDescription = stringResource(R.string.navigate),
                )
            }
        },
    )
}
