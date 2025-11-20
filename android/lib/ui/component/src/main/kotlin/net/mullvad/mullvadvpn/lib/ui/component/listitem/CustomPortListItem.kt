package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewCustomPortListItem() {
    AppTheme {
        SpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            CustomPortListItem(
                hierarchy = Hierarchy.Child1,
                title = "Custom",
                isSelected = true,
                port = Port(4444),
                onPortCellClicked = {},
                onMainCellClicked = {},
            )
            CustomPortListItem(
                hierarchy = Hierarchy.Child1,
                title = "Custom",
                isSelected = true,
                isEnabled = false,
                port = Port(44449),
                onPortCellClicked = {},
                onMainCellClicked = {},
            )
            CustomPortListItem(
                hierarchy = Hierarchy.Child1,
                title = "Custom",
                isSelected = false,
                port = null,
                onPortCellClicked = {},
                onMainCellClicked = {},
            )
        }
    }
}

@Composable
fun CustomPortListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    isEnabled: Boolean = true,
    isSelected: Boolean,
    port: Port?,
    mainTestTag: String = "",
    numberTestTag: String = "",
    onMainCellClicked: (() -> Unit)? = null,
    onPortCellClicked: () -> Unit,
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        isSelected = isSelected,
        testTag = mainTestTag,
        onClick = onMainCellClicked,
        content = {
            Column {
                Text(title)
                if (port != null) {
                    Text(
                        "Port: ${port.value}",
                        style = MaterialTheme.typography.labelLarge,
                        color =
                            if (isEnabled) MaterialTheme.colorScheme.onSurfaceVariant
                            else ListItemDefaults.colors().disabledHeadlineColor,
                    )
                }
            }
        },
        trailingContent = {
            DividerButton(
                modifier = Modifier.testTag(numberTestTag),
                onClick = onPortCellClicked,
                isEnabled = isEnabled,
                icon = Icons.Default.Edit,
            )
        },
    )
}
