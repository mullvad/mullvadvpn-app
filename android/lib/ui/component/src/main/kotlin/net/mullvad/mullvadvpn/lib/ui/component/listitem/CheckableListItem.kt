package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.times
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Checkbox
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Composable
@Preview
private fun PreviewCheckableListItem() {
    AppTheme {
        PreviewSpacedColumn {
            CheckableListItem(title = "Parent 1", isChecked = true, onCheckedChange = {})
            CheckableListItem(
                title = "Parent 2",
                isChecked = false,
                position = Position.Top,
                onCheckedChange = {},
            )
            CheckableListItem(
                title = "Child 1",
                isChecked = false,
                position = Position.Middle,
                hierarchy = Hierarchy.Child1,
                onCheckedChange = {},
            )
            CheckableListItem(
                title = "Child 2",
                isChecked = true,
                position = Position.Middle,
                hierarchy = Hierarchy.Child2,
                isEnabled = false,
                onCheckedChange = {},
            )
            CheckableListItem(
                title = "Child 2",
                isChecked = true,
                position = Position.Bottom,
                hierarchy = Hierarchy.Child2,
                onCheckedChange = {},
            )
        }
    }
}

@Composable
fun CheckableListItem(
    title: String,
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    isChecked: Boolean,
    isEnabled: Boolean = true,
    onCheckedChange: (isChecked: Boolean) -> Unit,
) {
    MullvadListItem(
        modifier = modifier,
        leadingContent = {
            Checkbox(
                modifier = Modifier.padding(end = Dimens.smallPadding),
                checked = isChecked,
                onCheckedChange = onCheckedChange,
            )
        },
        content = { Text(title) },
        position = position,
        hierarchy = hierarchy,
        isEnabled = isEnabled,
        onClick = { onCheckedChange(!isChecked) },
    )
}
