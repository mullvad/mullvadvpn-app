package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevronDivider
import net.mullvad.mullvadvpn.lib.ui.component.listitem.CheckableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Composable
@Preview
private fun PreviewCheckableRelayListItem(
    @PreviewParameter(RelayItemCheckableCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.map {
                Spacer(Modifier.size(1.dp))
                CheckableRelayListItem(
                    item = CheckableRelayListItem(item = it, itemPosition = Position.Single),
                    onExpand = {},
                    modifier = Modifier.testTag(LOCATION_CELL_TEST_TAG),
                )
            }
        }
    }
}

@Composable
fun CheckableRelayListItem(
    modifier: Modifier = Modifier,
    item: CheckableRelayListItem,
    onRelayCheckedChange: (isChecked: Boolean) -> Unit = { _ -> },
    onExpand: (Boolean) -> Unit,
) {

    CheckableListItem(
        modifier = modifier,
        hierarchy = item.hierarchy,
        position = item.itemPosition,
        title = item.item.name,
        isChecked = item.checked,
        onCheckedChange = { onRelayCheckedChange(!item.checked) },
        trailingContent = {
            if (item.item.hasChildren) {
                ExpandChevronDivider(
                    isExpanded = item.expanded,
                    modifier = Modifier.testTag(EXPAND_BUTTON_TEST_TAG),
                    onClick = { onExpand(!item.expanded) },
                )
            }
        },
    )
}
