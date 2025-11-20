package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Box
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
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.ComponentTokens
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewInfoListItem() {
    AppTheme {
        InfoListItem(
            title = "Information row title",
            isEnabled = true,
            onCellClicked = {},
            onInfoClicked = {},
        )
    }
}

@Composable
fun InfoListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    isEnabled: Boolean = true,
    backgroundAlpha: Float = 1f,
    onCellClicked: (() -> Unit)? = null,
    onInfoClicked: (() -> Unit),
    testTag: String = "",
) {

    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        onClick = onCellClicked,
        testTag = testTag,
        backgroundAlpha = backgroundAlpha,
        content = { Text(title) },
        trailingContent = {
            Box(
                modifier =
                    Modifier.width(ComponentTokens.infoIconContainerWidth)
                        .padding(end = Dimens.smallPadding)
                        .fillMaxHeight(),
                contentAlignment = Alignment.Center,
            ) {
                IconButton(onClick = onInfoClicked) {
                    Icon(imageVector = Icons.Default.Info, contentDescription = null)
                }
            }
        },
    )
}
