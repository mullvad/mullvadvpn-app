package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.MullvadSwitch
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewSwitchListItem() {
    AppTheme {
        SwitchListItem(
            title = "Checkbox Title",
            isEnabled = true,
            isToggled = true,
            onCellClicked = {},
            onInfoClicked = {},
        )
    }
}

@Composable
fun SwitchListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    isToggled: Boolean,
    isEnabled: Boolean = true,
    backgroundAlpha: Float = 1f,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        backgroundAlpha = backgroundAlpha,
        onClick = { onCellClicked(!isToggled) },
        content = { Text(title) },
        trailingContent = {
            Row(
                modifier = modifier.fillMaxHeight(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (onInfoClicked != null) {
                    Box(
                        modifier =
                            Modifier.width(ListItemComponentTokens.infoIconContainerWidth).fillMaxHeight(),
                        contentAlignment = Alignment.Center,
                    ) {
                        IconButton(onClick = onInfoClicked) {
                            Icon(
                                imageVector = Icons.Default.Info,
                                contentDescription = stringResource(id = R.string.more_information),
                            )
                        }
                    }
                }

                Box(modifier = Modifier.fillMaxHeight().padding(end = Dimens.smallPadding)) {
                    MullvadSwitch(
                        modifier = Modifier.align(Alignment.Center),
                        checked = isToggled,
                        onCheckedChange = onCellClicked,
                        enabled = isEnabled,
                    )
                }
            }
        },
    )
}
