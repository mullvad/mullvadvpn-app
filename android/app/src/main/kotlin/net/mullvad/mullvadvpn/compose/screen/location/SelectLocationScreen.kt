package net.mullvad.mullvadvpn.compose.screen.location

import android.annotation.SuppressLint
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.Velocity
import androidx.constraintlayout.compose.ExperimentalMotionApi
import androidx.constraintlayout.compose.MotionLayout
import androidx.constraintlayout.compose.MotionScene
import androidx.constraintlayout.compose.Visibility
import androidx.constraintlayout.compose.layoutId
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
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.SelectLocationsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.isTv
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.component.MultihopSelector
import net.mullvad.mullvadvpn.lib.ui.component.Singlehop
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.displayName
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.usecase.FilterChip
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
            onModifyMultihop = { _, _ -> },
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
            onRefreshRelayList = {},
            onSetAsExit = {},
            onSetAsEntry = {},
            setMultihop = { _, _ -> },
        )
    }
}

@SuppressLint("CheckResult")
@Destination<RootGraph>(style = TopLevelTransition::class)
@Suppress("LongMethod", "CyclomaticComplexMethod")
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
            is SelectLocationSideEffect.EntryAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            context.getString(
                                R.string.relay_item_already_selected_as_entry,
                                it.relayItem.name,
                            )
                    )
                }
            is SelectLocationSideEffect.ExitAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            context.getString(
                                R.string.relay_item_already_selected_as_exit,
                                it.relayItem.name,
                            )
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
            SelectLocationSideEffect.EntryAndExitAreSame ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.entry_and_exit_are_same)
                    )
                }
            SelectLocationSideEffect.RelayListUpdating ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.updating_server_list_in_the_background)
                    )
                }
            is SelectLocationSideEffect.MultihopChanged -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            context.getString(
                                if (it.enabled) {
                                    R.string.multihop_is_enabled
                                } else {
                                    R.string.multihop_is_disabled
                                }
                            ),
                        actionLabel = context.getString(R.string.undo),
                        onAction = { vm.setMultihop(!it.enabled) },
                        duration = SnackbarDuration.Long,
                    )
                }
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
            RelayListType.Single -> backNavigator.navigateBack(result = true)
            is RelayListType.Multihop ->
                when (result.multihopRelayListType) {
                    MultihopRelayListType.ENTRY -> vm.selectRelayList(MultihopRelayListType.EXIT)
                    MultihopRelayListType.EXIT -> backNavigator.navigateBack(result = true)
                }
        }
    }

    SelectLocationScreen(
        state = state.value,
        snackbarHostState = snackbarHostState,
        onSelectHop = vm::selectHop,
        onModifyMultihop = vm::modifyMultihop,
        onSearchClick =
            dropUnlessResumed { relayListType ->
                navigator.navigate(SearchLocationDestination(relayListType))
            },
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() },
        onFilterClick =
            dropUnlessResumed { relayListType ->
                navigator.navigate(FilterDestination(relayListType))
            },
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
        onRefreshRelayList = vm::refreshRelayList,
        onSetAsEntry = vm::setAsEntry,
        onSetAsExit = vm::setAsExit,
        setMultihop = vm::setMultihop,
    )
}

@Suppress("LongMethod", "LongParameterList")
@Composable
fun SelectLocationScreen(
    state: Lc<Unit, SelectLocationUiState>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectHop: (item: Hop) -> Unit,
    onModifyMultihop: (relayItem: RelayItem, relayListType: MultihopRelayListType) -> Unit,
    onSearchClick: (RelayListType) -> Unit,
    onBackClick: () -> Unit,
    onFilterClick: (RelayListType) -> Unit,
    onCreateCustomList: (location: RelayItem.Location?) -> Unit,
    onEditCustomLists: () -> Unit,
    onRecentsToggleEnableClick: () -> Unit,
    removeOwnershipFilter: (RelayListType) -> Unit,
    removeProviderFilter: (RelayListType) -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onSelectRelayList: (MultihopRelayListType) -> Unit,
    openDaitaSettings: () -> Unit,
    onRefreshRelayList: () -> Unit,
    onSetAsEntry: (RelayItem) -> Unit,
    onSetAsExit: (RelayItem) -> Unit,
    setMultihop: (enable: Boolean, showSnackbar: Boolean) -> Unit,
) {
    val backgroundColor = MaterialTheme.colorScheme.surface
    var fabHeight by remember { mutableIntStateOf(0) }
    val bottomMarginList = with(LocalDensity.current) { fabHeight.toDp() + Dimens.fabSpacing }

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
        floatingActionButton = {
            if (!isTv() && state is Lc.Content && state.value.isSearchButtonEnabled) {
                FloatingActionButton(
                    modifier = Modifier.onGloballyPositioned { fabHeight = it.size.height },
                    onClick = { onSearchClick(state.value.relayListType) },
                    containerColor = MaterialTheme.colorScheme.surface,
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                ) {
                    Icon(
                        imageVector = Icons.Default.Search,
                        contentDescription = stringResource(id = R.string.search),
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
        },
        actions = {
            if (isTv()) {
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
            }
            val recentsCurrentlyEnabled = state.contentOrNull()?.isRecentsEnabled == true
            val multihopEnabled = state.contentOrNull()?.multihopEnabled == true
            val disabledText = stringResource(id = R.string.recents_disabled)
            val scope = rememberCoroutineScope()

            SelectLocationDropdownMenu(
                recentsEnabled = recentsCurrentlyEnabled,
                multihopEnabled = multihopEnabled,
                onRecentsToggleEnableClick = {
                    if (recentsCurrentlyEnabled) {
                        scope.launch { snackbarHostState.showSnackbarImmediately(disabledText) }
                    }
                    onRecentsToggleEnableClick()
                },
                onRefreshRelayList = onRefreshRelayList,
                onMultihopToggleEnableClick = { setMultihop(!multihopEnabled, false) },
            )
        },
    ) { modifier ->
        var locationBottomSheetState by remember { mutableStateOf<LocationBottomSheetState?>(null) }
        LocationBottomSheets(
            locationBottomSheetState = locationBottomSheetState,
            enableEntryOption = state.contentOrNull()?.entrySelectionAllowed == true,
            onCreateCustomList = onCreateCustomList,
            onAddLocationToList = onAddLocationToList,
            onRemoveLocationFromList = onRemoveLocationFromList,
            onEditCustomListName = onEditCustomListName,
            onEditLocationsCustomList = onEditLocationsCustomList,
            onDeleteCustomList = onDeleteCustomList,
            onHideBottomSheet = { locationBottomSheetState = null },
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = { setMultihop(false, true) },
        )

        val expandProgress = remember { Animatable(1f) }
        val scope = rememberCoroutineScope()
        val nestedScrollConnection = remember {
            object : NestedScrollConnection {
                override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                    // TODO Calculate the value
                    val delta = available.y / 265f
                    scope.launch {
                        expandProgress.snapTo((expandProgress.value + delta).coerceIn(0f, 1f))
                    }
                    return super.onPreScroll(available, source)
                }

                override suspend fun onPostFling(
                    consumed: Velocity,
                    available: Velocity,
                ): Velocity {
                    scope.launch {
                        expandProgress.animateTo(if (expandProgress.value < 0.5f) 0f else 1f)
                    }
                    return super.onPostFling(consumed, available)
                }
            }
        }
        Column(
            modifier =
                modifier
                    .nestedScroll(nestedScrollConnection)
                    .background(backgroundColor)
                    .fillMaxSize(),
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
                    SelectionContainer(
                        progress = expandProgress.value,
                        relayListType = state.value.relayListType,
                        filterChips = state.value.filterChips,
                        entrySelection = state.value.entrySelection,
                        exitSelection = state.value.exitSelection,
                        error = state.value.tunnelErrorStateCause,
                        onSelectRelayList = onSelectRelayList,
                        onFilterClick = onFilterClick,
                        removeOwnershipFilter = removeOwnershipFilter,
                        removeProviderFilter = removeProviderFilter,
                    )

                    RelayLists(
                        state = state.value,
                        bottomMargin = bottomMarginList,
                        onSelectHop = onSelectHop,
                        onModifyMultihop = onModifyMultihop,
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
    recentsEnabled: Boolean,
    multihopEnabled: Boolean,
    onRecentsToggleEnableClick: () -> Unit,
    onRefreshRelayList: () -> Unit,
    onMultihopToggleEnableClick: () -> Unit,
) {
    var showMenu by remember { mutableStateOf(false) }

    var recentsItemTextId by remember { mutableIntStateOf(R.string.disable_recents) }
    var multihopItemTextId by remember { mutableIntStateOf(R.string.disable_multihop) }

    IconButton(
        onClick = {
            showMenu = !showMenu
            // Only update the recents and multihop menu items text when the menu is being opened to
            // prevent
            // the texts from being updated when the menu is being closed.
            if (showMenu) {
                recentsItemTextId =
                    if (recentsEnabled) R.string.disable_recents else R.string.enable_recents
                multihopItemTextId =
                    if (multihopEnabled) R.string.disable_multihop else R.string.enable_multihop
            }
        }
    ) {
        Icon(
            imageVector = Icons.Default.MoreVert,
            contentDescription = stringResource(R.string.more_actions),
        )
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
            text = { Text(text = stringResource(recentsItemTextId)) },
            onClick = {
                showMenu = false
                onRecentsToggleEnableClick()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Filled.History, contentDescription = null) },
        )

        DropdownMenuItem(
            text = { Text(text = stringResource(multihopItemTextId)) },
            onClick = {
                showMenu = false
                onMultihopToggleEnableClick()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Outlined.AddLocationAlt, contentDescription = null) },
        )

        DropdownMenuItem(
            text = { Text(text = stringResource(R.string.refresh_server_list)) },
            onClick = {
                showMenu = false
                onRefreshRelayList()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Filled.Refresh, contentDescription = null) },
        )
    }
}

@Composable
@Suppress("ComplexCondition")
private fun RelayLists(
    state: SelectLocationUiState,
    bottomMargin: Dp,
    onSelectHop: (hop: Hop) -> Unit,
    onModifyMultihop: (RelayItem, MultihopRelayListType) -> Unit,
    openDaitaSettings: () -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    val onSelectRelayItem: (RelayItem, RelayListType) -> Unit = { relayItem, relayListType ->
        if (relayListType is RelayListType.Multihop) {
            onModifyMultihop(relayItem, relayListType.multihopRelayListType)
        } else {
            onSelectHop(Hop.Single(relayItem))
        }
    }

    Crossfade(state.relayListType) {
        SelectLocationList(
            relayListType = it,
            bottomMargin = bottomMargin,
            onSelectHop = onSelectHop,
            onSelectRelayItem = onSelectRelayItem,
            openDaitaSettings = openDaitaSettings,
            onAddCustomList = onAddCustomList,
            onEditCustomLists = onEditCustomLists,
            onUpdateBottomSheetState = onUpdateBottomSheetState,
        )
    }
}

@OptIn(ExperimentalMotionApi::class)
@Suppress("LongMethod")
@Composable
private fun SelectionContainer(
    progress: Float, // 0 - 1
    relayListType: RelayListType,
    entrySelection: String?,
    exitSelection: String?,
    error: ErrorStateCause?,
    filterChips: Map<RelayListType, List<FilterChip>>,
    onSelectRelayList: (MultihopRelayListType) -> Unit,
    onFilterClick: (RelayListType) -> Unit,
    removeOwnershipFilter: (RelayListType) -> Unit,
    removeProviderFilter: (RelayListType) -> Unit,
) {
    Column {
        AnimatedContent(
            relayListType is RelayListType.Single,
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        ) {
            if (it) {
                Singlehop(
                    exitLocation = exitSelection ?: stringResource(R.string.any),
                    errorText = error.errorText(RelayListType.Single),
                    filters = filterChips[RelayListType.Single]?.size ?: 0,
                    onFilterClick = { onFilterClick(RelayListType.Single) },
                    expandProgress = progress,
                )
            } else {
                MultihopSelector(
                    exitSelected =
                        (relayListType as? RelayListType.Multihop)?.multihopRelayListType ==
                            MultihopRelayListType.EXIT,
                    exitLocation = exitSelection ?: stringResource(R.string.any),
                    exitErrorText =
                        error.errorText(RelayListType.Multihop(MultihopRelayListType.EXIT)),
                    exitFilters =
                        filterChips[RelayListType.Multihop(MultihopRelayListType.EXIT)]?.size ?: 0,
                    onExitClick = { onSelectRelayList(MultihopRelayListType.EXIT) },
                    onExitFilterClick = {
                        onFilterClick(RelayListType.Multihop(MultihopRelayListType.EXIT))
                    },
                    entryLocation = entrySelection ?: stringResource(R.string.any),
                    entryErrorText =
                        error.errorText(RelayListType.Multihop(MultihopRelayListType.ENTRY)),
                    entryFilters =
                        filterChips[RelayListType.Multihop(MultihopRelayListType.ENTRY)]?.size ?: 0,
                    onEntryClick = { onSelectRelayList(MultihopRelayListType.ENTRY) },
                    onEntryFilterClick = {
                        onFilterClick(RelayListType.Multihop(MultihopRelayListType.ENTRY))
                    },
                    expandProgress = progress,
                )
            }
        }

        val keyFilters = "filters"
        val scene = MotionScene {
            val expandSet =
                constraintSet("expanded") {
                    val filters = createRefFor(keyFilters)
                    constrain(filters) {
                        centerTo(parent)
                        visibility = Visibility.Visible
                    }
                }

            val collapseSet =
                constraintSet("collapsed") {
                    val filters = createRefFor(keyFilters)
                    constrain(filters) {
                        linkTo(start = parent.start, end = parent.end)
                        bottom.linkTo(parent.top)
                        visibility = Visibility.Gone
                    }
                }

            defaultTransition(collapseSet, expandSet) {}
        }
        MotionLayout(
            modifier =
                Modifier.padding(
                    bottom = Dimens.smallPadding,
                    start = Dimens.mediumPadding,
                    end = Dimens.mediumPadding,
                ),
            motionScene = scene,
            progress = progress,
        ) {
            AnimatedContent(
                relayListType,
                Modifier.animateContentSize().layoutId(keyFilters).alpha(progress),
            ) {
                FilterRow(
                    filters = filterChips[it] ?: emptyList(),
                    onRemoveOwnershipFilter = { removeOwnershipFilter(relayListType) },
                    onRemoveProviderFilter = { removeProviderFilter(relayListType) },
                )
            }
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}

@Composable
private fun ErrorStateCause?.errorText(relayListType: RelayListType) =
    when ((this as? ErrorStateCause.TunnelParameterError)?.error) {
        ParameterGenerationError.NoMatchingRelay if relayListType is RelayListType.Single -> {
            stringResource(R.string.no_matching_relay)
        }
        ParameterGenerationError.NoMatchingRelayEntry if
            relayListType is RelayListType.Multihop &&
                relayListType.multihopRelayListType == MultihopRelayListType.ENTRY
         -> {
            stringResource(R.string.no_matching_relay)
        }
        ParameterGenerationError.NoMatchingRelayExit if
            relayListType is RelayListType.Multihop &&
                relayListType.multihopRelayListType == MultihopRelayListType.EXIT
         -> {
            stringResource(R.string.no_matching_relay)
        }
        else -> null
    }
