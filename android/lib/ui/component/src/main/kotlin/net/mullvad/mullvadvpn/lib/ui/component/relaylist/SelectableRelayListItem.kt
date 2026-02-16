package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevronDivider
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemClickArea
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_LIST_ENTRY_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaVisible

@Composable
@Preview
private fun PreviewSelectableRelayLocationItem(
    @PreviewParameter(SelectableRelayListItemPreviewParameterProvider::class)
    relayItems: List<RelayListItem.SelectableItem>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.map {
                Spacer(Modifier.size(1.dp))
                SelectableRelayListItem(relayListItem = it, onClick = {}, onToggleExpand = {})
            }
        }
    }
}

@Composable
fun SelectableRelayListItem(
    modifier: Modifier = Modifier,
    relayListItem: RelayListItem.SelectableItem,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit),
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = relayListItem.hierarchy,
        position = relayListItem.itemPosition,
        isSelected = relayListItem.isSelected,
        title = relayListItem.item.name,
        iconContentDescription = null,
        mainClickArea =
            if (relayListItem.canExpand) ListItemClickArea.LeadingAndMain
            else ListItemClickArea.All,
        onClick = onClick,
        onLongClick = onLongClick,
        testTag =
            when (relayListItem) {
                is RelayListItem.CustomListEntryItem -> CUSTOM_LIST_ENTRY_NAME_TAG
                is RelayListItem.CustomListItem -> CUSTOM_LIST_ENTRY_NAME_TAG
                is RelayListItem.GeoLocationItem -> GEOLOCATION_NAME_TAG
                is RelayListItem.RecentListItem -> RECENT_NAME_TAG
            },
        trailingContent = {
            if (relayListItem.canExpand) {
                ExpandChevronDivider(
                    isExpanded = relayListItem.expanded,
                    modifier = Modifier.testTag(EXPAND_BUTTON_TEST_TAG),
                    onClick = { onToggleExpand(!relayListItem.expanded) },
                )
            }
        },
    )
}

@Composable
internal fun Name(
    modifier: Modifier = Modifier,
    name: String,
    state: RelayListItemState?,
    active: Boolean,
) {
    Text(
        text = state?.let { name.withSuffix(state) } ?: name,
        style = MaterialTheme.typography.bodyLarge,
        modifier =
            modifier.alpha(
                if (state == null && active) {
                    AlphaVisible
                } else {
                    AlphaInactive
                }
            ),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun String.withSuffix(state: RelayListItemState) =
    when (state) {
        RelayListItemState.USED_AS_EXIT -> stringResource(R.string.x_exit, this)
        RelayListItemState.USED_AS_ENTRY -> stringResource(R.string.x_entry, this)
    }
