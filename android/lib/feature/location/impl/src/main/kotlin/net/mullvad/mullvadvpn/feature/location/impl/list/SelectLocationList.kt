package net.mullvad.mullvadvpn.feature.location.impl.list

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.Unit
import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.impl.CustomListHeader
import net.mullvad.mullvadvpn.feature.location.impl.relayListContent
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.animateScrollCentralizeItem
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListTokens
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Content|Loading|Error")
@Composable
private fun PreviewSelectLocationList(
    @PreviewParameter(SelectLocationsListUiStatePreviewParameterProvider::class)
    state: Lce<Unit, SelectLocationListUiState, Unit>
) {
    AppTheme {
        Surface {
            SelectLocationList(
                state = state,
                bottomMargin = 0.dp,
                relayListType = RelayListType.Single,
                onSelectRelayItem = { _, _ -> },
                onSetMultihopToAlways = {},
                onSelectAutomaticEntry = {},
                onAutomaticInfoClick = {},
                onAddCustomList = {},
                onEditCustomLists = {},
                onUpdateBottomSheetState = {},
                onToggleExpand = { _, _, _ -> },
                lazyListStates = SnapshotStateMap(),
            )
        }
    }
}

private typealias EntryBlocked = Lce.Error<Unit>

private typealias Content = Lce.Content<SelectLocationListUiState>

@Composable
fun SelectLocationList(
    bottomMargin: Dp,
    relayListType: RelayListType,
    onSelectRelayItem: (RelayItem, RelayListType) -> Unit,
    onSetMultihopToAlways: () -> Unit,
    onSelectAutomaticEntry: () -> Unit,
    onAutomaticInfoClick: () -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
    lazyListStates: SnapshotStateMap<RelayListType, LazyListState>,
) {
    val viewModel =
        koinViewModel<SelectLocationListViewModel>(
            key = relayListType.toString(),
            parameters = { parametersOf(relayListType) },
        )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        val stateActual = viewModel.uiState.first { it is Content }
        // Ensure the selected item and its parents are expanded
        when (val id = it.relayItem.id) {
            is CustomListId,
            is GeoLocationId.Country -> viewModel.onToggleExpand(id, expand = true)
            is GeoLocationId.City -> {
                viewModel.onToggleExpand(id.country, expand = true)
                viewModel.onToggleExpand(id, expand = true)
            }
            is GeoLocationId.Hostname -> {
                viewModel.onToggleExpand(id.country, expand = true)
                viewModel.onToggleExpand(id.city, expand = true)
            }
        }
        val index =
            stateActual.contentOrNull()?.relayListItems?.indexOfFirst { relayListItem ->
                when (relayListItem) {
                    is RelayListItem.CustomListItem -> it.relayItem.id == relayListItem.item.id
                    is RelayListItem.GeoLocationItem -> it.relayItem.id == relayListItem.item.id
                    else -> false
                }
            }

        if (index != null && index != -1) {
            lazyListStates[relayListType]?.animateScrollCentralizeItem(index)
        }
    }

    SelectLocationList(
        state = state,
        bottomMargin = bottomMargin,
        relayListType = relayListType,
        onSelectRelayItem = onSelectRelayItem,
        onSetMultihopToAlways = onSetMultihopToAlways,
        onSelectAutomaticEntry = onSelectAutomaticEntry,
        onAutomaticInfoClick = onAutomaticInfoClick,
        onAddCustomList = onAddCustomList,
        onEditCustomLists = onEditCustomLists,
        onUpdateBottomSheetState = onUpdateBottomSheetState,
        onToggleExpand = { id, customListId, expand ->
            viewModel.onToggleExpand(id, customListId, expand)
        },
        lazyListStates = lazyListStates,
    )
}

@Composable
fun SelectLocationList(
    state: Lce<Unit, SelectLocationListUiState, Unit>,
    bottomMargin: Dp,
    relayListType: RelayListType,
    onSelectRelayItem: (RelayItem, RelayListType) -> Unit,
    onSetMultihopToAlways: () -> Unit,
    onSelectAutomaticEntry: () -> Unit,
    onAutomaticInfoClick: () -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    lazyListStates: SnapshotStateMap<RelayListType, LazyListState>,
) {
    when (state) {
        is Lce.Content -> {
            val density = LocalDensity.current
            val lazyListState =
                lazyListStates.getOrPut(relayListType) {
                    if (state.value.recentsEnabled) {
                        // If we have recents we start on the top of the list
                        rememberLazyListState()
                    } else {
                        // If no recents we focus the selected item
                        val selectedIndex = state.indexOfSelectedRelayItem()
                        val listItemHeight =
                            with(density) { ListTokens.listItemMinHeight.toPx().toInt() }
                        rememberLazyListState(selectedIndex ?: 0, (-2 * listItemHeight))
                    }
                }

            SelectLocationListContent(
                state = state.value,
                lazyListState = lazyListState,
                bottomMargin = bottomMargin,
                onSelectRelayItem = onSelectRelayItem,
                onSelectAutomaticEntry = onSelectAutomaticEntry,
                onAutomaticInfoClick = onAutomaticInfoClick,
                onUpdateBottomSheetState = onUpdateBottomSheetState,
                onAddCustomList = onAddCustomList,
                onEditCustomLists = onEditCustomLists,
                onToggleExpand = onToggleExpand,
            )
        }
        is Lce.Loading -> Loading()
        is EntryBlocked -> EntryBlocked(onSetMultihopToAlways = onSetMultihopToAlways)
    }
}

@Composable
private fun SelectLocationListContent(
    state: SelectLocationListUiState,
    lazyListState: LazyListState,
    bottomMargin: Dp,
    onSelectRelayItem: (relayItem: RelayItem, relayListType: RelayListType) -> Unit,
    onSelectAutomaticEntry: () -> Unit,
    onAutomaticInfoClick: () -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
) {
    var prevTopItem by remember { mutableStateOf<RelayListItem?>(null) }

    LazyColumn(
        modifier =
            Modifier.fillMaxSize()
                .padding(horizontal = Dimens.mediumPadding)
                .drawVerticalScrollbar(
                    lazyListState,
                    MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .testTag(SELECT_LOCATION_LIST_TEST_TAG),
        contentPadding = PaddingValues(bottom = bottomMargin),
        state = lazyListState,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // When recents have been disabled and are enabled again and we are at the
        // top of the list we scroll up so that recents are visible again.
        val shouldScrollToTop =
            state.relayListItems.firstOrNull() is RelayListItem.RecentsListHeader &&
                prevTopItem !is RelayListItem.RecentsListHeader &&
                lazyListState.firstVisibleItemIndex == 0 &&
                lazyListState.firstVisibleItemScrollOffset == 0

        prevTopItem = state.relayListItems.firstOrNull()

        relayListContent(
            relayListItems = state.relayListItems,
            relayListType = state.relayListType,
            onSelectRelayItem = { onSelectRelayItem(it, state.relayListType) },
            onSelectAutomaticEntry = onSelectAutomaticEntry,
            onAutomaticInfoClick = onAutomaticInfoClick,
            onToggleExpand = onToggleExpand,
            onUpdateBottomSheetState = onUpdateBottomSheetState,
            customListHeader = {
                CustomListHeader(onAddCustomList, if (it.canEdit) onEditCustomLists else null)
            },
        )

        if (shouldScrollToTop) {
            lazyListState.requestScrollToItem(0)
        }
    }
}

@Composable
private fun Loading() {
    Column(modifier = Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        MullvadCircularProgressIndicatorLarge()
    }
}

@Composable
private fun EntryBlocked(onSetMultihopToAlways: () -> Unit) {
    Column(modifier = Modifier.fillMaxSize(), horizontalAlignment = Alignment.CenterHorizontally) {
        Spacer(modifier = Modifier.weight(1f))
        Icon(
            modifier = Modifier.size(Dimens.dialogIconHeight),
            imageVector = MultihopWhenNeeded,
            tint = MaterialTheme.colorScheme.onSurface,
            contentDescription = null,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
        Text(
            text = stringResource(R.string.multihop_entry_disabled_description),
            style = MaterialTheme.typography.labelLarge,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        )
        Spacer(modifier = Modifier.weight(1f))
        PrimaryButton(
            text = stringResource(R.string.set_multihop_to_always),
            onClick = onSetMultihopToAlways,
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        )
        Spacer(modifier = Modifier.height(Dimens.screenBottomMargin))
    }
}

private fun Lce<Unit, SelectLocationListUiState, Unit>.indexOfSelectedRelayItem(): Int? =
    if (this is Content) {
        val index =
            value.relayListItems.indexOfFirst {
                when (it) {
                    is RelayListItem.CustomListItem -> it.isSelected
                    is RelayListItem.GeoLocationItem -> it.isSelected
                    is RelayListItem.RecentListItem -> it.isSelected
                    is RelayListItem.AutomaticEntryItem -> it.isSelected
                    is RelayListItem.CustomListEntryItem,
                    is RelayListItem.CustomListFooter,
                    is RelayListItem.CustomListHeader,
                    RelayListItem.LocationHeader,
                    is RelayListItem.LocationsEmptyText,
                    is RelayListItem.EmptyRelayList,
                    RelayListItem.RecentsListFooter,
                    RelayListItem.RecentsListHeader,
                    is RelayListItem.SectionDivider -> false
                }
            }
        if (index >= 0) index else null
    } else {
        null
    }
