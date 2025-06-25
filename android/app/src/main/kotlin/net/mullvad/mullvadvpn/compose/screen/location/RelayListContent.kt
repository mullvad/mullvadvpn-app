package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.StatusRelayItemCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.EmptyRelayListText
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowCustomListsEntryBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowEditCustomListBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowLocationBottomSheet
import net.mullvad.mullvadvpn.compose.state.ItemPosition
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
    relayListItems: List<RelayListItem>,
    customLists: List<RelayItem.CustomList>,
    onSelectRelay: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
    customListHeader: @Composable (LazyItemScope.() -> Unit) = {},
    locationHeader: @Composable (LazyItemScope.() -> Unit) = { RelayLocationHeader() },
) {
    items(
        items = relayListItems,
        key = { item: RelayListItem -> item.key },
        contentType = { item: RelayListItem -> item.contentType },
        itemContent = { listItem: RelayListItem ->
            Column(modifier = Modifier.animateItem()) {
                when (listItem) {
                    RelayListItem.CustomListHeader -> customListHeader()
                    is RelayListItem.CustomListItem ->
                        CustomListItem(
                            listItem,
                            onSelectRelay,
                            { onUpdateBottomSheetState(ShowEditCustomListBottomSheet(it)) },
                            { customListId, expand -> onToggleExpand(customListId, null, expand) },
                            modifier = Modifier.positionalPadding(listItem.itemPosition),
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
                            modifier = Modifier.positionalPadding(listItem.itemPosition),
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
                            modifier = Modifier.positionalPadding(listItem.itemPosition),
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
fun Modifier.positionalPadding(itemPosition: ItemPosition): Modifier =
    when (itemPosition) {
        ItemPosition.Top,
        ItemPosition.Single -> padding(top = Dimens.miniPadding)
        ItemPosition.Middle -> padding(top = Dimens.listItemDivider)
        ItemPosition.Bottom -> padding(top = Dimens.listItemDivider, bottom = Dimens.miniPadding)
    }

@Composable
private fun RelayLocationItem(
    relayItem: RelayListItem.GeoLocationItem,
    onSelectRelay: () -> Unit,
    onLongClick: () -> Unit,
    onExpand: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    StatusRelayItemCell(
        item = relayItem.item,
        state = relayItem.state,
        itemPosition = relayItem.itemPosition,
        isSelected = relayItem.isSelected,
        onClick = { onSelectRelay() },
        onLongClick = { onLongClick() },
        onToggleExpand = { onExpand(it) },
        isExpanded = relayItem.expanded,
        depth = relayItem.depth,
        modifier = modifier.testTag(LOCATION_CELL_TEST_TAG),
    )
}

@Composable
fun Modifier.clip(itemPosition: ItemPosition): Modifier =
    clip(
        with(MaterialTheme.shapes.large) {
            val topCornerSize =
                animateDpAsState(
                    if (itemPosition.roundTop()) Dimens.relayItemCornerRadius else 0.dp
                )
            val bottomCornerSize =
                animateDpAsState(
                    if (itemPosition.roundBottom()) Dimens.relayItemCornerRadius else 0.dp
                )
            copy(
                topStart = CornerSize(topCornerSize.value),
                topEnd = CornerSize(topCornerSize.value),
                bottomStart = CornerSize(bottomCornerSize.value),
                bottomEnd = CornerSize(bottomCornerSize.value),
            )
        }
    )

@Composable
private fun CustomListEntryItem(
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
        itemPosition = itemState.itemPosition,
        isSelected = false,
        onClick = onSelectRelay,
        onLongClick = onShowEditCustomListEntryBottomSheet,
        onToggleExpand = onToggleExpand,
        isExpanded = itemState.expanded,
        depth = itemState.depth,
        modifier = modifier,
    )
}

@Composable
private fun CustomListItem(
    itemState: RelayListItem.CustomListItem,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowEditBottomSheet: (RelayItem.CustomList) -> Unit,
    onExpand: (CustomListId, Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val customListItem = itemState.item
    StatusRelayItemCell(
        item = customListItem,
        state = itemState.state,
        itemPosition = itemState.itemPosition,
        isSelected = itemState.isSelected,
        onClick = { onSelectRelay(customListItem) },
        onLongClick = { onShowEditBottomSheet(customListItem) },
        onToggleExpand = { onExpand(customListItem.id, it) },
        isExpanded = itemState.expanded,
        modifier = modifier,
    )
}

@Composable
fun CustomListHeader(addCustomList: () -> Unit, editCustomLists: (() -> Unit)?) {
    RelayListHeader(
        { Text(stringResource(R.string.custom_lists), overflow = TextOverflow.Ellipsis) },
        actions = {
            IconButton(onClick = addCustomList) {
                Icon(
                    imageVector = Icons.Default.Add,
                    contentDescription = stringResource(id = R.string.new_list),
                )
            }
            editCustomLists?.run {
                IconButton(onClick = editCustomLists) {
                    Icon(
                        imageVector = Icons.Default.Edit,
                        contentDescription = stringResource(id = R.string.edit_lists),
                    )
                }
            }
        },
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG),
    )
}

@Composable
private fun CustomListFooter(item: RelayListItem.CustomListFooter) {
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
private fun RelayLocationHeader() {
    RelayListHeader(
        content = {
            Text(text = stringResource(R.string.all_locations), overflow = TextOverflow.Ellipsis)
        }
    )
}
