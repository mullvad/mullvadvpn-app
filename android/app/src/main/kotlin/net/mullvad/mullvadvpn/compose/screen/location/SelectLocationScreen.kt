package net.mullvad.mullvadvpn.compose.screen.location

import android.annotation.SuppressLint
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.animateScrollBy
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.generated.destinations.FilterDestination
import com.ramcosta.composedestinations.generated.destinations.SearchLocationDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.result.onResult
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.MullvadSegmentedButton
import net.mullvad.mullvadvpn.compose.button.SegmentedButtonPosition
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.SelectLocationsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.RunOnKeyChange
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Loading")
@Composable
private fun PreviewSelectLocationScreen(
    @PreviewParameter(SelectLocationsUiStatePreviewParameterProvider::class)
    state: SelectLocationUiState
) {
    AppTheme { SelectLocationScreen(state = state) }
}

@SuppressLint("CheckResult")
@Destination<RootGraph>(style = SelectLocationTransition::class)
@Suppress("LongMethod")
@Composable
fun SelectLocation(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<Boolean>,
    createCustomListDialogResultRecipient:
        ResultRecipient<
            CreateCustomListDestination,
            CustomListActionResultData.Success.CreatedWithLocations,
        >,
    editCustomListNameDialogResultRecipient:
        ResultRecipient<EditCustomListNameDestination, CustomListActionResultData.Success.Renamed>,
    deleteCustomListDialogResultRecipient:
        ResultRecipient<DeleteCustomListDestination, CustomListActionResultData.Success.Deleted>,
    updateCustomListResultRecipient:
        ResultRecipient<CustomListLocationsDestination, CustomListActionResultData>,
    searchSelectedLocationResultRecipient:
        ResultRecipient<SearchLocationDestination, RelayListSelection>,
) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            SelectLocationSideEffect.CloseScreen -> backNavigator.navigateBack(result = true)
            is SelectLocationSideEffect.CustomListActionToast ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.resultData,
                        onUndo = vm::performAction,
                    )
                }
            SelectLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.error_occurred),
                        duration = SnackbarDuration.Short,
                    )
                }
        }
    }

    createCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction,
    )

    editCustomListNameDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction,
    )

    deleteCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction,
    )

    updateCustomListResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    searchSelectedLocationResultRecipient.onResult { result ->
        when (result) {
            RelayListSelection.Entry -> {
                vm.selectRelayList(RelayListSelection.Exit)
            }
            RelayListSelection.Exit -> backNavigator.navigateBack(result = true)
        }
    }

    SelectLocationScreen(
        state = state.value,
        snackbarHostState = snackbarHostState,
        onSelectRelay = vm::selectRelay,
        onSearchClick = { navigator.navigate(SearchLocationDestination(it)) },
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() },
        onFilterClick = dropUnlessResumed { navigator.navigate(FilterDestination) },
        onCreateCustomList =
            dropUnlessResumed { relayItem ->
                navigator.navigate(CreateCustomListDestination(locationCode = relayItem?.id))
            },
        onToggleExpand = vm::onToggleExpand,
        onEditCustomLists = dropUnlessResumed { navigator.navigate(CustomListsDestination()) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
        onAddLocationToList = vm::addLocationToList,
        onRemoveLocationFromList = vm::removeLocationFromList,
        onEditCustomListName =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigate(
                    EditCustomListNameDestination(
                        customListId = customList.id,
                        initialName = customList.customList.name,
                    )
                )
            },
        onEditLocationsCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigate(
                    CustomListLocationsDestination(customListId = customList.id, newList = false)
                )
            },
        onDeleteCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigate(
                    DeleteCustomListDestination(
                        customListId = customList.id,
                        name = customList.customList.name,
                    )
                )
            },
        onSelectRelayList = vm::selectRelayList,
    )
}

@Suppress("LongMethod")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchClick: (RelayListSelection) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    onCreateCustomList: (location: RelayItem.Location?) -> Unit = {},
    onEditCustomLists: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit =
        { _, _ ->
        },
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit =
        { _, _ ->
        },
    onEditCustomListName: (RelayItem.CustomList) -> Unit = {},
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit = {},
    onDeleteCustomList: (RelayItem.CustomList) -> Unit = {},
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit = { _, _, _ -> },
    onSelectRelayList: (RelayListSelection) -> Unit = {},
) {
    val backgroundColor = MaterialTheme.colorScheme.surface

    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.select_location),
        navigationIcon = {
            IconButton(onClick = onBackClick) {
                Icon(
                    imageVector = Icons.Default.Close,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = stringResource(id = R.string.back),
                )
            }
        },
        snackbarHostState = snackbarHostState,
        actions = {
            IconButton(
                onClick = {
                    if (state is SelectLocationUiState.Content) {
                        onSearchClick(state.relayListSelection)
                    }
                },
                enabled = state is SelectLocationUiState.Content,
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    contentDescription = stringResource(id = R.string.filter),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
            IconButton(onClick = onFilterClick) {
                Icon(
                    imageVector = Icons.Default.FilterList,
                    contentDescription = stringResource(id = R.string.filter),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        },
    ) { modifier ->
        var locationBottomSheetState by remember { mutableStateOf<LocationBottomSheetState?>(null) }
        LocationBottomSheets(
            locationBottomSheetState = locationBottomSheetState,
            onCreateCustomList = onCreateCustomList,
            onEditCustomLists = onEditCustomLists,
            onAddLocationToList = onAddLocationToList,
            onRemoveLocationFromList = onRemoveLocationFromList,
            onEditCustomListName = onEditCustomListName,
            onEditLocationsCustomList = onEditLocationsCustomList,
            onDeleteCustomList = onDeleteCustomList,
            onHideBottomSheet = { locationBottomSheetState = null },
        )

        Column(modifier = modifier.background(backgroundColor).fillMaxSize()) {
            if (state is SelectLocationUiState.Content && state.filterChips.isNotEmpty()) {
                FilterRow(
                    filters = state.filterChips,
                    onRemoveOwnershipFilter = removeOwnershipFilter,
                    onRemoveProviderFilter = removeProviderFilter,
                )
            }

            if (state is SelectLocationUiState.Content && state.multihopEnabled) {
                MultihopBar(state.relayListSelection, onSelectRelayList)
            }

            Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
            RelayList(
                state = state,
                backgroundColor = backgroundColor,
                onSelectRelay = onSelectRelay,
                onToggleExpand = onToggleExpand,
                onUpdateBottomSheetState = { newState -> locationBottomSheetState = newState },
            )
        }
    }
}

@Composable
private fun MultihopBar(
    relayListSelection: RelayListSelection,
    onSelectRelayList: (RelayListSelection) -> Unit,
) {
    SingleChoiceSegmentedButtonRow(
        modifier =
            Modifier.fillMaxWidth().padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
    ) {
        MullvadSegmentedButton(
            selected = relayListSelection == RelayListSelection.Entry,
            onClick = { onSelectRelayList(RelayListSelection.Entry) },
            text = stringResource(id = R.string.enter),
            position = SegmentedButtonPosition.First,
        )
        MullvadSegmentedButton(
            selected = relayListSelection == RelayListSelection.Exit,
            onClick = { onSelectRelayList(RelayListSelection.Exit) },
            text = stringResource(id = R.string.exit),
            position = SegmentedButtonPosition.Last,
        )
    }
}

@Composable
private fun RelayList(
    state: SelectLocationUiState,
    backgroundColor: Color,
    onSelectRelay: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    val pagerState = rememberPagerState(pageCount = { 2 })
    LaunchedEffect(state.relayListSelection()) {
        val index = state.relayListSelection()?.ordinal ?: 0
        pagerState.animateScrollToPage(index)
    }

    HorizontalPager(state = pagerState, userScrollEnabled = false) { pageIndex ->
        val lazyListState = rememberLazyListState()
        val stateActual = state
        RunOnKeyChange(stateActual is SelectLocationUiState.Content) {
            val index = stateActual.indexOfSelectedRelayItem()
            if (index != -1) {
                lazyListState.scrollToItem(index)
                lazyListState.animateScrollAndCentralizeItem(index)
            }
        }
        LazyColumn(
            modifier =
                Modifier.fillMaxSize()
                    .drawVerticalScrollbar(
                        lazyListState,
                        MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    ),
            state = lazyListState,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            when (state) {
                SelectLocationUiState.Loading -> {
                    loading()
                }
                is SelectLocationUiState.Content -> {
                    relayListContent(
                        backgroundColor = backgroundColor,
                        relayListItems = state.relayListItems,
                        customLists = state.customLists,
                        relayListSelection = state.relayListSelection,
                        onSelectRelay = onSelectRelay,
                        onToggleExpand = onToggleExpand,
                        onUpdateBottomSheetState = onUpdateBottomSheetState,
                    )
                }
            }
        }
    }
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR))
    }
}

private fun SelectLocationUiState.indexOfSelectedRelayItem(): Int =
    if (this is SelectLocationUiState.Content) {
        relayListItems.indexOfFirst {
            when (it) {
                is RelayListItem.CustomListItem -> it.isSelected
                is RelayListItem.GeoLocationItem -> it.isSelected
                is RelayListItem.CustomListEntryItem -> false
                is RelayListItem.CustomListFooter -> false
                RelayListItem.CustomListHeader -> false
                RelayListItem.LocationHeader -> false
                is RelayListItem.LocationsEmptyText -> false
            }
        }
    } else {
        -1
    }

private suspend fun LazyListState.animateScrollAndCentralizeItem(index: Int) {
    val itemInfo = this.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }
    if (itemInfo != null) {
        val center = layoutInfo.viewportEndOffset / 2
        val childCenter = itemInfo.offset + itemInfo.size / 2
        animateScrollBy((childCenter - center).toFloat())
    } else {
        animateScrollToItem(index)
    }
}
