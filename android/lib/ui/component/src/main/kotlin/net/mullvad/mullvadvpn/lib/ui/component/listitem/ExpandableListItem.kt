package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.ComponentTokens
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG

@Preview
@Composable
private fun PreviewExpandedEnabledExpandableListItem() {
    AppTheme {
        ExpandableListItem(
            title = "Expandable row title",
            isExpanded = true,
            isEnabled = true,
            onCellClicked = {},
            onInfoClicked = {},
        )
    }
}

@Composable
fun ExpandableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        onClick = { onCellClicked(!isExpanded) },
        content = { Text(title) },
        trailingContent = {
            Row(modifier = modifier.fillMaxSize()) {
                if (onInfoClicked != null) {
                    Box(
                        modifier =
                            Modifier.width(ComponentTokens.infoIconContainerWidth).fillMaxHeight(),
                        contentAlignment = Alignment.Center,
                    ) {
                        IconButton(onClick = onInfoClicked) {
                            Icon(imageVector = Icons.Default.Info, contentDescription = null)
                        }
                    }
                }
                Box(
                    modifier = Modifier.width(ChevronContainerWidth).fillMaxHeight(),
                    contentAlignment = Alignment.Center,
                ) {
                    ExpandChevron(
                        isExpanded = isExpanded,
                        modifier =
                            Modifier.padding(end = ChevronIconPaddingEnd)
                                .testTag(EXPAND_BUTTON_TEST_TAG),
                    )
                }
            }
        },
    )
}

private val ChevronContainerWidth = 60.dp
private val ChevronIconPaddingEnd = 8.dp
