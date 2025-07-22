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
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
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
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.displayName
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Default|Filters|Multihop|Multihop and Filters")
@Composable
private fun PreviewSelectLocationScreen(
    @PreviewParameter(SelectLocationsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, SelectLocationUiState>
) {
    AppTheme {
        SelectLocationScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onSelectHop = {},
            onSearchClick = {},
            onBackClick = {},
            onFilterClick = {},
            onCreateCustomList = { _ -> },
            onEditCustomLists = {},
            onRecentsToggleEnableClick = {},
            removeOwnershipFilter = {},
            removeProviderFilter = {},
            onAddLocationToList = { _, _ -> },
            onRemoveLocationFromList = { _, _ -> },
            onEditCustomListName = {},
            onEditLocationsCustomList = {},
            onDeleteCustomList = {},
            onSelectRelayList = {},
            openDaitaSettings = {},
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

            is SelectLocationSideEffect.RelayItemInactive ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            context.getString(
                                R.string.relayitem_is_inactive,
                                it.hop.displayName(context),
                            )
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
        onSelectHop = vm::selectHop,
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
        onRecentsToggleEnableClick = vm::toggleRecentsEnabled,
        openDaitaSettings =
            dropUnlessResumed { navigator.navigate(DaitaDestination(isModal = true)) },
    )
}

@Suppress("LongMethod", "LongParameterList")
@Composable
fun SelectLocationScreen(
    state: Lc<Unit, SelectLocationUiState>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectHop: (item: Hop) -> Unit,
    onSearchClick: (RelayListType) -> Unit,
    onBackClick: () -> Unit,
    onFilterClick: () -> Unit,
    onCreateCustomList: (location: RelayItem.Location?) -> Unit,
    onEditCustomLists: () -> Unit,
    onRecentsToggleEnableClick: () -> Unit,
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
            val isSearchButtonEnabled = state.contentOrNull()?.isSearchButtonEnabled == true
            IconButton(
                enabled = isSearchButtonEnabled,
                onClick = { state.contentOrNull()?.let { onSearchClick(it.relayListType) } },
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    contentDescription = stringResource(id = R.string.search),
                    tint =
                        MaterialTheme.colorScheme.onSurface.copy(
                            alpha = if (isSearchButtonEnabled) AlphaVisible else AlphaDisabled
                        ),
                )
            }

            val filterButtonEnabled = state.contentOrNull()?.isFilterButtonEnabled == true
            val recentsEnabled = state.contentOrNull()?.isRecentsEnabled == true

            SelectLocationDropdownMenu(
                filterButtonEnabled = filterButtonEnabled,
                onFilterClick = onFilterClick,
                recentsEnabled = recentsEnabled,
                onRecentsToggleEnableClick = onRecentsToggleEnableClick,
            )
        },
    ) { modifier ->
        var locationBottomSheetState by remember { mutableStateOf<LocationBottomSheetState?>(null) }
        LocationBottomSheets(
            locationBottomSheetState = locationBottomSheetState,
            onCreateCustomList = onCreateCustomList,
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
                    is Lc.Loading -> Arrangement.Center
                    is Lc.Content -> Arrangement.Top
                },
        ) {
            when (state) {
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    if (state.value.multihopEnabled) {
                        MultihopBar(state.value.relayListType, onSelectRelayList)
                    }

                    AnimatedContent(
                        targetState = state.value.filterChips,
                        label = "Select location top bar",
                    ) { filterChips ->
                        if (filterChips.isNotEmpty()) {
                            FilterRow(
                                modifier = Modifier.padding(bottom = Dimens.smallPadding),
                                filters = filterChips,
                                onRemoveOwnershipFilter = removeOwnershipFilter,
                                onRemoveProviderFilter = removeProviderFilter,
                            )
                        }
                    }

                    if (state.value.multihopEnabled && state.value.filterChips.isEmpty()) {
                        Spacer(modifier = Modifier.height(Dimens.smallPadding))
                    }

                    RelayLists(
                        state = state.value,
                        onSelectHop = onSelectHop,
                        openDaitaSettings = openDaitaSettings,
                        onAddCustomList = { onCreateCustomList(null) },
                        onEditCustomLists = onEditCustomLists,
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
private fun SelectLocationDropdownMenu(
    filterButtonEnabled: Boolean,
    onFilterClick: () -> Unit,
    recentsEnabled: Boolean,
    onRecentsToggleEnableClick: () -> Unit,
) {
    var showMenu by remember { mutableStateOf(false) }

    var recentsItemTextId by remember { mutableIntStateOf(R.string.disable_recents) }

    IconButton(
        onClick = {
            showMenu = !showMenu
            // Only update the recents menu item text when the menu is being opened to prevent
            // the text from being updated when the menu is being closed.
            if (showMenu) {
                recentsItemTextId =
                    if (recentsEnabled) R.string.disable_recents else R.string.enable_recents
            }
        }
    ) {
        Icon(imageVector = Icons.Default.MoreVert, contentDescription = null)
    }
    DropdownMenu(
        modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer),
        expanded = showMenu,
        onDismissRequest = { showMenu = false },
    ) {
        val colors =
            MenuDefaults.itemColors(
                leadingIconColor = MaterialTheme.colorScheme.onPrimary,
                disabledLeadingIconColor =
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
            )

        DropdownMenuItem(
            text = { Text(text = stringResource(R.string.filter)) },
            onClick = {
                onFilterClick()
                showMenu = false
            },
            enabled = filterButtonEnabled,
            colors = colors,
            leadingIcon = { Icon(Icons.Filled.FilterList, contentDescription = null) },
        )

        DropdownMenuItem(
            text = { Text(text = stringResource(recentsItemTextId)) },
            onClick = {
                showMenu = false
                onRecentsToggleEnableClick()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Filled.History, contentDescription = null) },
        )
    }
}

@Composable
private fun MultihopBar(relayListType: RelayListType, onSelectHopList: (RelayListType) -> Unit) {
    SingleChoiceSegmentedButtonRow(
        modifier =
            Modifier.fillMaxWidth()
                .padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.smallPadding,
                )
    ) {
        MullvadSegmentedStartButton(
            selected = relayListType == RelayListType.ENTRY,
            onClick = { onSelectHopList(RelayListType.ENTRY) },
            text = stringResource(id = R.string.entry),
        )
        MullvadSegmentedEndButton(
            selected = relayListType == RelayListType.EXIT,
            onClick = { onSelectHopList(RelayListType.EXIT) },
            text = stringResource(id = R.string.exit),
        )
    }
}

@Composable
private fun RelayLists(
    state: SelectLocationUiState,
    onSelectHop: (Hop) -> Unit,
    openDaitaSettings: () -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    // This is a workaround for the HorizontalPager being broken on Android TV when it contains
    // focusable views and you navigate with the D-pad. Remove this code once DROID-1639 is fixed.
    val configuration = LocalConfiguration.current

    if (configuration.navigation == Configuration.NAVIGATION_DPAD) {
        SelectLocationList(
            relayListType = state.relayListType,
            onSelectHop = onSelectHop,
            openDaitaSettings = openDaitaSettings,
            onAddCustomList = onAddCustomList,
            onEditCustomLists = onEditCustomLists,
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
                relayListType = RelayListType.entries[pageIndex],
                onSelectHop = onSelectHop,
                openDaitaSettings = openDaitaSettings,
                onAddCustomList = onAddCustomList,
                onEditCustomLists = onEditCustomLists,
                onUpdateBottomSheetState = onUpdateBottomSheetState,
            )
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}
