package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.EmptyRelayListText
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowCustomListsEntryBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowEditCustomListBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowLocationBottomSheet
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.SelectableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListHeader
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG

/** Used by both the select location screen and search select location screen */
fun LazyListScope.relayListContent(
    relayListItems: List<RelayListItem>,
    relayListType: RelayListType,
    onSelectRelayItem: (RelayItem) -> Unit,
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
                            listItem = listItem,
                            relayListType = relayListType,
                            onSelect = onSelectRelayItem,
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                        )
                    is RelayListItem.CustomListEntryItem ->
                        CustomListEntryItem(
                            listItem = listItem,
                            relayListType = relayListType,
                            onSelect = onSelectRelayItem,
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                        )
                    is RelayListItem.CustomListFooter -> CustomListFooter(listItem)
                    RelayListItem.LocationHeader -> locationHeader()
                    is RelayListItem.GeoLocationItem ->
                        GeoLocationItem(
                            listItem = listItem,
                            relayListType = relayListType,
                            onSelect = onSelectRelayItem,
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                        )

                    RelayListItem.RecentsListHeader -> RecentsListHeader()
                    is RelayListItem.RecentListItem ->
                        RecentListItem(
                            listItem = listItem,
                            relayListType = relayListType,
                            onSelect = onSelectRelayItem,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                        )
                    RelayListItem.RecentsListFooter -> RecentsListFooter()
                    is RelayListItem.EmptyRelayList -> EmptyRelayListText()
                    is RelayListItem.LocationsEmptyText -> LocationsEmptyText(listItem.searchTerm)
                    is RelayListItem.SectionDivider -> SectionDivider()
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
private fun GeoLocationItem(
    listItem: RelayListItem.GeoLocationItem,
    relayListType: RelayListType,
    onSelect: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    SelectableRelayListItem(
        relayListItem = listItem,
        onClick = { onSelect(listItem.item) },
        onLongClick = {
            onUpdateBottomSheetState(
                ShowLocationBottomSheet(item = listItem.item, relayListType = relayListType)
            )
        },
        onToggleExpand = { onToggleExpand(listItem.item.id, null, it) },
        modifier = Modifier.positionalPadding(listItem.itemPosition).testTag(LOCATION_CELL_TEST_TAG),
    )
}

@Composable
private fun RecentListItem(
    listItem: RelayListItem.RecentListItem,
    relayListType: RelayListType,
    onSelect: (RelayItem) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    SelectableRelayListItem(
        relayListItem = listItem,
        onClick = { onSelect(listItem.item) },
        onLongClick = {
            when (val entry = listItem.item) {
                is RelayItem.CustomList ->
                    onUpdateBottomSheetState(
                        ShowEditCustomListBottomSheet(item = entry, relayListType = relayListType)
                    )
                is RelayItem.Location ->
                    onUpdateBottomSheetState(
                        ShowLocationBottomSheet(item = entry, relayListType = relayListType)
                    )
            }
        },
        onToggleExpand = { _ -> },
        modifier = Modifier.positionalPadding(listItem.itemPosition).testTag(RECENT_CELL_TEST_TAG),
    )
}

@Composable
private fun CustomListItem(
    listItem: RelayListItem.CustomListItem,
    relayListType: RelayListType,
    onSelect: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    SelectableRelayListItem(
        relayListItem = listItem,
        onClick = { onSelect(listItem.item) },
        onLongClick = {
            onUpdateBottomSheetState(
                ShowEditCustomListBottomSheet(item = listItem.item, relayListType = relayListType)
            )
        },
        onToggleExpand = { onToggleExpand(listItem.item.id, null, it) },
        modifier = Modifier.positionalPadding(listItem.itemPosition),
    )
}

@Composable
private fun CustomListEntryItem(
    listItem: RelayListItem.CustomListEntryItem,
    relayListType: RelayListType,
    onSelect: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    SelectableRelayListItem(
        relayListItem = listItem,
        onClick = { onSelect(listItem.item) },
        // Only direct children can be removed
        onLongClick =
            if (listItem.depth == 1) {
                {
                    onUpdateBottomSheetState(
                        ShowCustomListsEntryBottomSheet(
                            customListId = listItem.parentId,
                            item = listItem.item,
                            relayListType = relayListType,
                        )
                    )
                }
            } else {
                null
            },
        onToggleExpand = { expand: Boolean ->
            onToggleExpand(listItem.item.id, listItem.parentId, expand)
        },
        modifier = Modifier.positionalPadding(listItem.itemPosition),
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
            }
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

@Composable
private fun RecentsListHeader() {
    RelayListHeader(
        content = {
            Text(text = stringResource(id = R.string.recents), overflow = TextOverflow.Ellipsis)
        }
    )
}

@Composable
private fun RecentsListFooter() {
    SwitchComposeSubtitleCell(text = stringResource(R.string.no_recent_selection))
}

@Composable
private fun SectionDivider() {
    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
}
