package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.times
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCheckbox
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_CELL_TEST_TAG

@Composable
@Preview
private fun PreviewCheckableRelayLocationCell(
    @PreviewParameter(RelayItemCheckableCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.map {
                Spacer(Modifier.size(1.dp))
                CheckableRelayLocationCell(
                    item = CheckableRelayListItem(item = it, itemPosition = ItemPosition.Single),
                    onExpand = {},
                    modifier = Modifier.testTag(LOCATION_CELL_TEST_TAG),
                )
            }
        }
    }
}

@Composable
fun CheckableRelayLocationCell(
    item: CheckableRelayListItem,
    modifier: Modifier = Modifier,
    onRelayCheckedChange: (isChecked: Boolean) -> Unit = { _ -> },
    onExpand: (Boolean) -> Unit,
) {
    RelayListItem(
        modifier = modifier.clip(itemPosition = item.itemPosition),
        selected = false,
        content = {
            Row(
                modifier =
                    Modifier.padding(start = item.depth * Dimens.mediumPadding)
                        .padding(Dimens.mediumPadding),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Name(name = item.item.name, state = null, active = item.item.active)
            }
        },
        leadingContent = {
            MullvadCheckbox(checked = item.checked, onCheckedChange = onRelayCheckedChange)
        },
        onClick = { onRelayCheckedChange(!item.checked) },
        onLongClick = null,
        trailingContent = {
            if (item.item.hasChildren) {
                ExpandChevron(
                    color = MaterialTheme.colorScheme.onSurface,
                    isExpanded = item.expanded,
                    modifier =
                        Modifier.clickable { onExpand(!item.expanded) }
                            .fillMaxSize()
                            .padding(Dimens.mediumPadding)
                            .testTag(EXPAND_BUTTON_TEST_TAG),
                )
            }
        },
        colors = RelayListItemDefaults.colors(containerColor = item.depth.toBackgroundColor()),
    )
}
