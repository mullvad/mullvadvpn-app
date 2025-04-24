package net.mullvad.mullvadvpn.compose.screen.location

import android.annotation.SuppressLint
import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import com.ramcosta.composedestinations.generated.destinations.DaitaDestination
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
import net.mullvad.mullvadvpn.compose.button.MullvadSegmentedEndButton
import net.mullvad.mullvadvpn.compose.button.MullvadSegmentedStartButton
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.SelectLocationsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel
import androidx.compose.ui.platform.LocalConfiguration

@Preview("Loading|Default|Filters|Multihop|Multihop and Filters")
@Composable
private fun PreviewSelectLocationScreen(
    @PreviewParameter(SelectLocationsUiStatePreviewParameterProvider::class)
    state: SelectLocationUiState
) {
    AppTheme {
        SelectLocationScreen(
            state = state,
            SnackbarHostState(),
            {},
            {},
            {},
            {},
            {},
            {},
            {},
            {},
            { _, _ -> },
            { _, _ -> },
            {},
            {},
            {},
            {},
            {},
        )
    }
}

@SuppressLint("CheckResult")
@Destination<RootGraph>(style = TopLevelTransition::class)
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
    searchSelectedLocationResultRecipient: ResultRecipient<SearchLocationDestination, RelayListType>,
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
                        message = context.getString(R.string.error_occurred)
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
            RelayListType.ENTRY -> {
                vm.selectRelayList(RelayListType.EXIT)
            }
            RelayListType.EXIT -> backNavigator.navigateBack(result = true)
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
        openDaitaSettings =
            dropUnlessResumed { navigator.navigate(DaitaDestination(isModal = true)) },
    )
}

@Suppress("LongMethod", "LongParameterList")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (item: RelayItem) -> Unit,
    onSearchClick: (RelayListType) -> Unit,
    onBackClick: () -> Unit,
    onFilterClick: () -> Unit,
    onCreateCustomList: (location: RelayItem.Location?) -> Unit,
    onEditCustomLists: () -> Unit,
    removeOwnershipFilter: () -> Unit,
    removeProviderFilter: () -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onSelectRelayList: (RelayListType) -> Unit,
    openDaitaSettings: () -> Unit,
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
        modifier = Modifier.testTag(SELECT_LOCATION_SCREEN_TEST_TAG),
        snackbarHostState = snackbarHostState,
        actions = {
            IconButton(
                enabled = state is SelectLocationUiState.Data,
                onClick = {
                    if (state is SelectLocationUiState.Data) onSearchClick(state.relayListType)
                },
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    contentDescription = stringResource(id = R.string.search),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
            IconButton(enabled = state is SelectLocationUiState.Data, onClick = onFilterClick) {
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

        Column(
            modifier = modifier.background(backgroundColor).fillMaxSize(),
            verticalArrangement =
                when (state) {
                    SelectLocationUiState.Loading -> Arrangement.Center
                    is SelectLocationUiState.Data -> Arrangement.Top
                },
        ) {
            when (state) {
                SelectLocationUiState.Loading -> {
                    Loading()
                }
                is SelectLocationUiState.Data -> {
                    AnimatedContent(
                        targetState = state.filterChips,
                        label = "Select location top bar",
                    ) { filterChips ->
                        if (filterChips.isNotEmpty()) {
                            FilterRow(
                                filters = filterChips,
                                onRemoveOwnershipFilter = removeOwnershipFilter,
                                onRemoveProviderFilter = removeProviderFilter,
                            )
                        }
                    }

                    if (state.multihopEnabled) {
                        MultihopBar(state.relayListType, onSelectRelayList)
                    }

                    if (state.filterChips.isNotEmpty() || state.multihopEnabled) {
                        Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
                    }

                    RelayLists(
                        state = state,
                        backgroundColor = backgroundColor,
                        onSelectRelay = onSelectRelay,
                        openDaitaSettings = openDaitaSettings,
                        onUpdateBottomSheetState = { newState ->
                            locationBottomSheetState = newState
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun MultihopBar(relayListType: RelayListType, onSelectRelayList: (RelayListType) -> Unit) {
    SingleChoiceSegmentedButtonRow(
        modifier =
            Modifier.fillMaxWidth().padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
    ) {
        MullvadSegmentedStartButton(
            selected = relayListType == RelayListType.ENTRY,
            onClick = { onSelectRelayList(RelayListType.ENTRY) },
            text = stringResource(id = R.string.entry),
        )
        MullvadSegmentedEndButton(
            selected = relayListType == RelayListType.EXIT,
            onClick = { onSelectRelayList(RelayListType.EXIT) },
            text = stringResource(id = R.string.exit),
        )
    }
}

@Composable
private fun RelayLists(
    state: SelectLocationUiState.Data,
    backgroundColor: Color,
    onSelectRelay: (RelayItem) -> Unit,
    openDaitaSettings: () -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    // This is a workaround for the HorizontalPager being broken on Android TV when it contains
    // focusable views and you navigate with the D-pad. Remove this code once DROID-1639 is fixed.
    val configuration = LocalConfiguration.current
    if (configuration.navigation == Configuration.NAVIGATION_DPAD) {
        SelectLocationList(
            backgroundColor = backgroundColor,
            relayListType = state.relayListType,
            onSelectRelay = onSelectRelay,
            openDaitaSettings = openDaitaSettings,
            onUpdateBottomSheetState = onUpdateBottomSheetState,
        )
    } else {
        val pagerState =
            rememberPagerState(
                initialPage = state.relayListType.ordinal,
                pageCount = { RelayListType.entries.size },
            )
        LaunchedEffect(state.relayListType) {
            val index = state.relayListType.ordinal
            pagerState.animateScrollToPage(index)
        }

        HorizontalPager(
            state = pagerState,
            userScrollEnabled = false,
            beyondViewportPageCount =
                if (state.multihopEnabled) {
                    1
                } else {
                    0
                },
        ) { pageIndex ->
            SelectLocationList(
                backgroundColor = backgroundColor,
                relayListType = RelayListType.entries[pageIndex],
                onSelectRelay = onSelectRelay,
                openDaitaSettings = openDaitaSettings,
                onUpdateBottomSheetState = onUpdateBottomSheetState,
            )
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}
