package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevronDivider
import net.mullvad.mullvadvpn.lib.ui.component.listitem.LeadingContentAnimatedVisibility
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemClickArea
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_LIST_ENTRY_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_LIST_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
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
    val active = relayListItem.item.active
    val selected = relayListItem.isSelected

    MullvadListItem(
        modifier = modifier,
        hierarchy = relayListItem.hierarchy,
        position = relayListItem.itemPosition,
        isSelected = selected,
        isEnabled = active,
        testTag =
            when (relayListItem) {
                is RelayListItem.CustomListEntryItem -> CUSTOM_LIST_ENTRY_ITEM_TAG
                is RelayListItem.CustomListItem -> CUSTOM_LIST_ITEM_TAG
                is RelayListItem.GeoLocationItem -> GEOLOCATION_ITEM_TAG
                is RelayListItem.RecentListItem -> RECENT_ITEM_TAG
            },
        mainClickArea =
            if (relayListItem.canExpand) ListItemClickArea.LeadingAndMain
            else ListItemClickArea.All,
        onClick = onClick,
        onLongClick = onLongClick,
        leadingContent = {
            LeadingContentAnimatedVisibility(
                modifier = Modifier.align(Alignment.Center),
                visible = selected || !active,
            ) {
                if (selected) {
                    Icon(
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        imageVector = Icons.Rounded.Check,
                        contentDescription = null,
                        tint =
                            if (!active) MaterialTheme.colorScheme.error
                            else LocalContentColor.current,
                    )
                } else if (!active) {
                    InactiveRelayIndicator(
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        tint = MaterialTheme.colorScheme.error,
                    )
                }
            }
        },
        content = {
            Name(
                name = relayListItem.item.name,
                state = relayListItem.state,
                active = relayListItem.item.active,
            )
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

@Composable
fun InactiveRelayIndicator(modifier: Modifier = Modifier, tint: Color) {
    Box(
        modifier =
            modifier
                .size(Dimens.listIconSize)
                .padding(Dimens.relayCirclePadding)
                .background(color = tint, shape = CircleShape)
    )
}
