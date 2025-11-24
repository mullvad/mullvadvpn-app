package net.mullvad.mullvadvpn.compose.screen.location

import android.annotation.SuppressLint
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.EaseInQuint
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material.icons.outlined.WrongLocation
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateMapOf
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
import androidx.compose.ui.unit.dp
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
import kotlin.math.roundToInt
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.SelectLocationsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.isTv
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.resource.icon.DeleteHistory
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.component.MultihopSelector
import net.mullvad.mullvadvpn.lib.ui.component.Singlehop
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

val SCROLL_COLLAPSE_DISTANCE = 150.dp
const val ANIMATION_DELAY_FADE_IN = 90

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
            onSelectSinglehop = {},
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
            toggleMultihop = {},
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
                            context.getString(R.string.relayitem_is_inactive, it.relayItem.name)
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
        onSelectSinglehop = vm::selectSingle,
        onModifyMultihop = vm::modifyMultihop,
        onSearchClick =
            dropUnlessResumed { relayListType ->
                navigator.navigate(SearchLocationDestination(relayListType))
            },
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
        onRefreshRelayList = vm::refreshRelayList,
        toggleMultihop = vm::toggleMultihop,
    )
}

@Suppress("LongMethod", "LongParameterList", "CyclomaticComplexMethod")
@Composable
fun SelectLocationScreen(
    state: Lc<Unit, SelectLocationUiState>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectSinglehop: (item: RelayItem) -> Unit,
    onModifyMultihop: (relayItem: RelayItem, relayListType: MultihopRelayListType) -> Unit,
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
    onSelectRelayList: (MultihopRelayListType) -> Unit,
    openDaitaSettings: () -> Unit,
    onRefreshRelayList: () -> Unit,
    toggleMultihop: (enable: Boolean) -> Unit,
) {
    val backgroundColor = MaterialTheme.colorScheme.surface
    var fabHeight by remember { mutableIntStateOf(0) }
    val bottomMarginList =
        if (isTv()) {
            0.dp
        } else {
            with(LocalDensity.current) { fabHeight.toDp() + Dimens.fabSpacing }
        }

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
            val filterButtonEnabled = state.contentOrNull()?.isFilterButtonEnabled == true
            val recentsCurrentlyEnabled = state.contentOrNull()?.isRecentsEnabled == true
            val multihopEnabled = state.contentOrNull()?.multihopEnabled == true
            val disabledText = stringResource(id = R.string.recents_disabled)
            val scope = rememberCoroutineScope()

            SelectLocationDropdownMenu(
                filterButtonEnabled = filterButtonEnabled,
                onFilterClick = onFilterClick,
                recentsEnabled = recentsCurrentlyEnabled,
                multihopEnabled = multihopEnabled,
                onRecentsToggleEnableClick = {
                    if (recentsCurrentlyEnabled) {
                        scope.launch { snackbarHostState.showSnackbarImmediately(disabledText) }
                    }
                    onRecentsToggleEnableClick()
                },
                onRefreshRelayList = onRefreshRelayList,
                onMultihopToggleEnableClick = { toggleMultihop(!multihopEnabled) },
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

        val expandProgress = remember { Animatable(1f) }

        val scope = rememberCoroutineScope()
        val scrollRequired = with(LocalDensity.current) { SCROLL_COLLAPSE_DISTANCE.toPx() }

        val nestedScrollConnection = remember {
            object : NestedScrollConnection {
                override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                    val delta = available.y / scrollRequired
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
                        expandProgress.animateTo(
                            expandProgress.value.roundToInt().toFloat(),
                            animationSpec = tween(),
                        )
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
                        hopSelection = state.value.hopSelection,
                        error = state.value.tunnelErrorStateCause,
                        onSelectRelayList = onSelectRelayList,
                        removeOwnershipFilter = removeOwnershipFilter,
                        removeProviderFilter = removeProviderFilter,
                    )

                    RelayLists(
                        relayListType = state.value.relayListType,
                        bottomMargin = bottomMarginList,
                        onSelect = onSelectSinglehop,
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
    filterButtonEnabled: Boolean,
    onFilterClick: () -> Unit,
    recentsEnabled: Boolean,
    multihopEnabled: Boolean,
    onRecentsToggleEnableClick: () -> Unit,
    onRefreshRelayList: () -> Unit,
    onMultihopToggleEnableClick: () -> Unit,
) {
    var showMenu by remember { mutableStateOf(false) }

    IconButton(onClick = { showMenu = !showMenu }) {
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
            text = { Text(text = stringResource(R.string.filter)) },
            onClick = {
                showMenu = false
                onFilterClick()
            },
            enabled = filterButtonEnabled,
            colors = colors,
            leadingIcon = { Icon(Icons.Filled.FilterList, contentDescription = null) },
        )

        // Keep these assets in remember so we don't change them as we animate away the dropdown
        // menu
        var recentsItemTextId by remember {
            mutableIntStateOf(
                if (recentsEnabled) R.string.disable_recents else R.string.enable_recents
            )
        }
        var recentsIcon by remember {
            mutableStateOf(if (recentsEnabled) DeleteHistory else Icons.Filled.History)
        }
        DropdownMenuItem(
            text = { Text(text = stringResource(recentsItemTextId)) },
            onClick = {
                showMenu = false
                onRecentsToggleEnableClick()
            },
            colors = colors,
            leadingIcon = { Icon(imageVector = recentsIcon, contentDescription = null) },
        )

        // Keep these assets in remember so we don't change them as we animate away the dropdown
        // menu
        var multihopItemTextId by remember {
            mutableIntStateOf(
                if (multihopEnabled) R.string.disable_multihop else R.string.enable_multihop
            )
        }
        var multihopIcon by remember {
            mutableStateOf(
                if (multihopEnabled) Icons.Outlined.WrongLocation else Icons.Outlined.AddLocationAlt
            )
        }
        DropdownMenuItem(
            text = { Text(text = stringResource(multihopItemTextId)) },
            onClick = {
                showMenu = false
                onMultihopToggleEnableClick()
            },
            colors = colors,
            leadingIcon = { Icon(multihopIcon, contentDescription = null) },
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
    relayListType: RelayListType,
    bottomMargin: Dp,
    onSelect: (item: RelayItem) -> Unit,
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
            onSelect(relayItem)
        }
    }

    val lazyListStates = remember { mutableStateMapOf<RelayListType, LazyListState>() }
    val scrollToLists = remember { mutableSetOf<RelayListType>() }

    Crossfade(relayListType) {
        when (it) {
            is RelayListType.Multihop ->
                when (it.multihopRelayListType) {
                    MultihopRelayListType.ENTRY ->
                        SelectLocationList(
                            relayListType = it,
                            bottomMargin = bottomMargin,
                            onSelectRelayItem = onSelectRelayItem,
                            openDaitaSettings = openDaitaSettings,
                            onAddCustomList = onAddCustomList,
                            onEditCustomLists = onEditCustomLists,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                            lazyListState =
                                lazyListStates.getOrPut(it, { rememberLazyListState() }),
                            scrollToList = scrollToLists.add(it),
                        )
                    MultihopRelayListType.EXIT ->
                        SelectLocationList(
                            relayListType = it,
                            bottomMargin = bottomMargin,
                            onSelectRelayItem = onSelectRelayItem,
                            openDaitaSettings = openDaitaSettings,
                            onAddCustomList = onAddCustomList,
                            onEditCustomLists = onEditCustomLists,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                            lazyListState =
                                lazyListStates.getOrPut(it, { rememberLazyListState() }),
                            scrollToList = scrollToLists.add(it),
                        )
                }
            RelayListType.Single ->
                SelectLocationList(
                    relayListType = it,
                    bottomMargin = bottomMargin,
                    onSelectRelayItem = onSelectRelayItem,
                    openDaitaSettings = openDaitaSettings,
                    onAddCustomList = onAddCustomList,
                    onEditCustomLists = onEditCustomLists,
                    onUpdateBottomSheetState = onUpdateBottomSheetState,
                    lazyListState = lazyListStates.getOrPut(it, { rememberLazyListState() }),
                    scrollToList = scrollToLists.add(it),
                )
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Suppress("LongMethod")
@Composable
private fun SelectionContainer(
    progress: Float, // 0 - 1
    relayListType: RelayListType,
    hopSelection: HopSelection,
    error: ErrorStateCause?,
    filterChips: List<FilterChip>,
    onSelectRelayList: (MultihopRelayListType) -> Unit,
    removeOwnershipFilter: () -> Unit,
    removeProviderFilter: () -> Unit,
) {

    var multihopListSelector by remember { mutableStateOf(MultihopRelayListType.EXIT) }
    if (relayListType is RelayListType.Multihop) {
        multihopListSelector = relayListType.multihopRelayListType
    }

    Column {
        AnimatedContent(
            hopSelection,
            contentKey = { it is HopSelection.Multi },
            transitionSpec = {
                fadeIn(tween(delayMillis = ANIMATION_DELAY_FADE_IN)).togetherWith(fadeOut())
            },
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        ) { hopSelection ->
            when (hopSelection) {
                is HopSelection.Single ->
                    Singlehop(
                        exitLocation = hopSelection.relay.toDisplayName(),
                        errorText = error.errorText(RelayListType.Single),
                        expandProgress = progress,
                    )
                is HopSelection.Multi ->
                    MultihopSelector(
                        exitSelected = multihopListSelector == MultihopRelayListType.EXIT,
                        exitLocation = hopSelection.exit.toDisplayName(),
                        exitErrorText =
                            error.errorText(RelayListType.Multihop(MultihopRelayListType.EXIT)),
                        onExitClick = { onSelectRelayList(MultihopRelayListType.EXIT) },
                        entryLocation = hopSelection.entry.toDisplayName(),
                        entryErrorText =
                            error.errorText(RelayListType.Multihop(MultihopRelayListType.ENTRY)),
                        onEntryClick = { onSelectRelayList(MultihopRelayListType.ENTRY) },
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
                        visibility = Visibility.Invisible
                    }
                }

            defaultTransition(collapseSet, expandSet) {}
        }
        MotionLayout(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            motionScene = scene,
            progress = progress,
        ) {
            FilterRow(
                modifier = Modifier.layoutId(keyFilters).alpha(EaseInQuint.transform(progress)),
                filters = filterChips,
                onRemoveOwnershipFilter = { removeOwnershipFilter() },
                onRemoveProviderFilter = { removeProviderFilter() },
            )
        }
    }
}

@Composable
fun Constraint<RelayItem>?.toDisplayName() =
    when (this) {
        Constraint.Any -> stringResource(R.string.automatic)
        is Constraint.Only<RelayItem> -> value.name
        null -> stringResource(R.string.unavailable)
    }

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}

@Composable
private fun ErrorStateCause?.errorText(relayListType: RelayListType) =
    when ((this as? ErrorStateCause.TunnelParameterError)?.error) {
        ParameterGenerationError.NoMatchingRelay if relayListType is RelayListType.Single ->
            stringResource(R.string.no_matching_relay)

        ParameterGenerationError.NoMatchingRelayEntry if
            relayListType is RelayListType.Multihop &&
                relayListType.multihopRelayListType == MultihopRelayListType.ENTRY
         -> stringResource(R.string.no_matching_relay)

        ParameterGenerationError.NoMatchingRelayExit if
            relayListType is RelayListType.Multihop &&
                relayListType.multihopRelayListType == MultihopRelayListType.EXIT
         -> stringResource(R.string.no_matching_relay)

        else -> null
    }
