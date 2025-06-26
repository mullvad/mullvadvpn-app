package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.StatusRelayItemCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.EmptyRelayListText
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowCustomListsBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowCustomListsEntryBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowEditCustomListBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowLocationBottomSheet
import net.mullvad.mullvadvpn.compose.state.PositionClassification
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListHeader
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG

/** Used by both the select location screen and search select location screen */
fun LazyListScope.relayListContent(
    backgroundColor: Color,
    relayListItems: List<RelayListItem>,
    customLists: List<RelayItem.CustomList>,
    onSelectRelay: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
    customListHeader: @Composable LazyItemScope.() -> Unit = {
        CustomListHeader(
            onShowCustomListBottomSheet = {
                onUpdateBottomSheetState(
                    ShowCustomListsBottomSheet(editListEnabled = customLists.isNotEmpty())
                )
            }
        )
    },
    locationHeader: @Composable LazyItemScope.() -> Unit = { RelayLocationHeader() },
) {
    itemsIndexed(
        items = relayListItems,
        key = { _: Int, item: RelayListItem -> item.key },
        contentType = { _, item -> item.contentType },
        itemContent = { index: Int, listItem: RelayListItem ->
            Column(modifier = Modifier.animateItem()) {
                when (listItem) {
                    RelayListItem.CustomListHeader -> customListHeader()
                    is RelayListItem.CustomListItem ->
                        CustomListItem(
                            listItem,
                            onSelectRelay,
                            { onUpdateBottomSheetState(ShowEditCustomListBottomSheet(it)) },
                            { customListId, expand -> onToggleExpand(customListId, null, expand) },
                            modifier = Modifier.positionalPadding(listItem.positionClassification),
                        )
                    is RelayListItem.CustomListEntryItem ->
                        CustomListEntryItem(
                            listItem,
                            { onSelectRelay(listItem.item) },
                            // Only direct children can be removed
                            if (listItem.depth == 1) {
                                {
                                    onUpdateBottomSheetState(
                                        ShowCustomListsEntryBottomSheet(
                                            listItem.parentId,
                                            listItem.parentName,
                                            listItem.item,
                                        )
                                    )
                                }
                            } else {
                                null
                            },
                            { expand: Boolean ->
                                onToggleExpand(listItem.item.id, listItem.parentId, expand)
                            },
                            modifier = Modifier.positionalPadding(listItem.positionClassification),
                        )
                    is RelayListItem.CustomListFooter -> CustomListFooter(listItem)
                    RelayListItem.LocationHeader -> locationHeader()
                    is RelayListItem.GeoLocationItem -> {
                        RelayLocationItem(
                            listItem,
                            { onSelectRelay(listItem.item) },
                            {
                                onUpdateBottomSheetState(
                                    ShowLocationBottomSheet(customLists, listItem.item)
                                )
                            },
                            { expand -> onToggleExpand(listItem.item.id, null, expand) },
                            modifier = Modifier.positionalPadding(listItem.positionClassification),
                        )
                    }
                    is RelayListItem.LocationsEmptyText -> LocationsEmptyText(listItem.searchTerm)
                    is RelayListItem.EmptyRelayList -> EmptyRelayListText()
                }
            }
        },
    )
}

@Composable
fun Modifier.positionalPadding(positionClassification: PositionClassification): Modifier =
    when (positionClassification) {
        PositionClassification.Top,
        PositionClassification.Single -> padding(top = Dimens.miniPadding)
        PositionClassification.Middle -> padding(top = Dimens.listItemDivider)
        PositionClassification.Bottom ->
            padding(top = Dimens.listItemDivider, bottom = Dimens.miniPadding)
    }

@Composable
private fun LazyItemScope.RelayLocationItem(
    relayItem: RelayListItem.GeoLocationItem,
    onSelectRelay: () -> Unit,
    onLongClick: () -> Unit,
    onExpand: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val location = relayItem.item
    StatusRelayItemCell(
        item = location,
        state = relayItem.state,
        isSelected = relayItem.isSelected,
        onClick = { onSelectRelay() },
        onLongClick = { onLongClick() },
        onToggleExpand = { onExpand(it) },
        isExpanded = relayItem.expanded,
        depth = relayItem.depth,
        modifier =
            modifier
                .testTag(LOCATION_CELL_TEST_TAG)
                .clip(positionClassification = relayItem.positionClassification),
    )
}

@Composable
fun Modifier.clip(positionClassification: PositionClassification): Modifier =
    clip(
        with(MaterialTheme.shapes.large) {
            val topCornerSize =
                animateDpAsState(if (positionClassification.roundTop()) 16.dp else 0.dp)
            val bottomCornerSize =
                animateDpAsState(if (positionClassification.roundBottom()) 16.dp else 0.dp)
            copy(
                topStart = CornerSize(topCornerSize.value),
                topEnd = CornerSize(topCornerSize.value),
                bottomStart = CornerSize(bottomCornerSize.value),
                bottomEnd = CornerSize(bottomCornerSize.value),
            )
        }
    )

@Composable
private fun LazyItemScope.CustomListEntryItem(
    itemState: RelayListItem.CustomListEntryItem,
    onSelectRelay: () -> Unit,
    onShowEditCustomListEntryBottomSheet: (() -> Unit)?,
    onToggleExpand: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val customListEntryItem = itemState.item
    StatusRelayItemCell(
        item = customListEntryItem,
        state = itemState.state,
        isSelected = false,
        onClick = onSelectRelay,
        onLongClick = onShowEditCustomListEntryBottomSheet,
        onToggleExpand = onToggleExpand,
        isExpanded = itemState.expanded,
        depth = itemState.depth,
        modifier = modifier.clip(positionClassification = itemState.positionClassification),
    )
}

@Composable
private fun LazyItemScope.CustomListItem(
    itemState: RelayListItem.CustomListItem,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowEditBottomSheet: (RelayItem.CustomList) -> Unit,
    onExpand: ((CustomListId, Boolean) -> Unit),
    modifier: Modifier = Modifier,
) {
    val customListItem = itemState.item
    StatusRelayItemCell(
        item = customListItem,
        state = itemState.state,
        isSelected = itemState.isSelected,
        onClick = { onSelectRelay(customListItem) },
        onLongClick = { onShowEditBottomSheet(customListItem) },
        onToggleExpand = { onExpand(customListItem.id, it) },
        isExpanded = itemState.expanded,
        modifier = modifier.clip(positionClassification = itemState.positionClassification),
    )
}

@Composable
private fun LazyItemScope.CustomListHeader(onShowCustomListBottomSheet: () -> Unit) {
    RelayListHeader(
        { Text(stringResource(R.string.custom_lists), overflow = TextOverflow.Ellipsis) },
        actions = {
            IconButton(onClick = onShowCustomListBottomSheet) {
                Icon(
                    imageVector = Icons.Default.MoreVert,
                    contentDescription = stringResource(id = R.string.custom_lists),
                )
            }
        },
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG),
    )
}

@Composable
private fun LazyItemScope.CustomListFooter(item: RelayListItem.CustomListFooter) {
    SwitchComposeSubtitleCell(
        text =
            if (item.hasCustomList) {
                stringResource(R.string.to_add_locations_to_a_list)
            } else {
                stringResource(R.string.to_create_a_custom_list)
            },
        modifier = Modifier.background(MaterialTheme.colorScheme.surface),
    )
}

@Composable
private fun LazyItemScope.RelayLocationHeader() {
    RelayListHeader(
        content = {
            Text(text = stringResource(R.string.all_locations), overflow = TextOverflow.Ellipsis)
        }
    )
}
