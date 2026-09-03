package net.mullvad.mullvadvpn.feature.location.impl

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
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material.icons.outlined.WrongLocation
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
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
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
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
import kotlin.math.roundToInt
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CreateCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.CreateCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.CustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.DeleteCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.UpdateCustomListNavResult
import net.mullvad.mullvadvpn.feature.filter.api.FilterNavKey
import net.mullvad.mullvadvpn.feature.location.api.AutomaticEntryInfoNavKey
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetNavKey
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetNavResult
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.api.SearchLocationNavKey
import net.mullvad.mullvadvpn.feature.location.api.SearchLocationNavResult
import net.mullvad.mullvadvpn.feature.location.api.SelectLocationNavResult
import net.mullvad.mullvadvpn.feature.location.api.UndoChangeMultihopAction
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.showResultSnackbar
import net.mullvad.mullvadvpn.feature.location.impl.list.SelectLocationList
import net.mullvad.mullvadvpn.feature.location.impl.list.SelectLocationListUiState
import net.mullvad.mullvadvpn.feature.location.impl.navigation.navigateFromFilterChip
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.lib.common.compose.isTv
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.hopType
import net.mullvad.mullvadvpn.lib.ui.component.FilterState
import net.mullvad.mullvadvpn.lib.ui.component.MultihopSelector
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.Singlehop
import net.mullvad.mullvadvpn.lib.ui.component.button.SearchButton
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemPreviewData
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.icon.DeleteHistory
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_MENU_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SEARCH_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive
import net.mullvad.mullvadvpn.lib.usecase.FilterChip
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
            onSearchClick = {},
            onBackClick = {},
            onFilterClick = {},
            onRecentsToggleEnableClick = {},
            removeOwnershipFilter = {},
            removeProviderFilter = {},
            onFilterChipNavigate = {},
            onSelectRelayList = {},
            onRefreshRelayList = {},
            scrollToItem = {},
            onSetMultihopMode = {},
            relayListContent = { relayListType, bottomMargin ->
                SelectLocationList(
                    state =
                        Lce.Content(
                            SelectLocationListUiState(
                                relayListItems =
                                    RelayListItemPreviewData.generateRelayListItems(
                                        includeCustomLists = true,
                                        isSearching = false,
                                    ),
                                relayListType = relayListType,
                                recentsEnabled = true,
                            )
                        ),
                    bottomMargin = bottomMargin,
                    relayListType = relayListType,
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
            },
        )
    }
}

@SuppressLint("CheckResult")
@Suppress("LongMethod", "CyclomaticComplexMethod")
@Composable
fun SelectLocation(navigator: Navigator) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    val resultStore = LocalResultStore.current

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            SelectLocationSideEffect.CloseScreen ->
                navigator.goBack(result = SelectLocationNavResult(true))

            SelectLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.error_occurred)
                    )
                }

            is SelectLocationSideEffect.EntryAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(
                                R.string.relay_item_already_selected_as_entry,
                                it.relayItem.name,
                            )
                    )
                }

            is SelectLocationSideEffect.ExitAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(
                                R.string.relay_item_already_selected_as_exit,
                                it.relayItem.name,
                            )
                    )
                }

            is SelectLocationSideEffect.RelayItemInactive ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(R.string.relayitem_is_inactive, it.relayItem.name)
                    )
                }

            SelectLocationSideEffect.EntryAndExitAreSame ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.entry_and_exit_are_same)
                    )
                }

            SelectLocationSideEffect.RelayListUpdating ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(R.string.updating_server_list_in_the_background)
                    )
                }
        }
    }

    resultStore.consumeResult<LocationBottomSheetNavResult> { result ->
        when (result) {
            is LocationBottomSheetNavResult.CustomListActionToast ->
                snackbarHostState.showResultSnackbar(
                    resources = resources,
                    result = result.resultData,
                    onUndo = vm::performAction,
                )

            LocationBottomSheetNavResult.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.error_occurred)
                )

            is LocationBottomSheetNavResult.EntryAlreadySelected ->
                snackbarHostState.showSnackbarImmediately(
                    message =
                        resources.getString(
                            R.string.relay_item_already_selected_as_entry,
                            result.relayItem.name,
                        )
                )

            is LocationBottomSheetNavResult.ExitAlreadySelected ->
                snackbarHostState.showSnackbarImmediately(
                    message =
                        resources.getString(
                            R.string.relay_item_already_selected_as_exit,
                            result.relayItem.name,
                        )
                )

            is LocationBottomSheetNavResult.RelayItemInactive ->
                snackbarHostState.showSnackbarImmediately(
                    message =
                        resources.getString(R.string.relayitem_is_inactive, result.relayItem.name)
                )

            LocationBottomSheetNavResult.EntryAndExitAreSame ->
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.entry_and_exit_are_same)
                )

            is LocationBottomSheetNavResult.MultihopChanged -> {
                snackbarHostState.showSnackbarImmediately(
                    message =
                        resources.getString(
                            when (result.undoChangeMultihopAction) {
                                UndoChangeMultihopAction.Disable,
                                is UndoChangeMultihopAction.DisableAndSetExit,
                                is UndoChangeMultihopAction.DisableAndSetEntry ->
                                    R.string.multihop_is_enabled

                                else -> R.string.multihop_is_disabled
                            }
                        ),
                    actionLabel = resources.getString(R.string.undo),
                    onAction = { vm.undoMultihopAction(result.undoChangeMultihopAction) },
                    duration = SnackbarDuration.Long,
                )
            }
        }
    }

    resultStore.consumeResult<CreateCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = vm::performAction,
        )
    }

    resultStore.consumeResult<EditCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = vm::performAction,
        )
    }

    resultStore.consumeResult<DeleteCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = vm::performAction,
        )
    }

    resultStore.consumeResult<UpdateCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = vm::performAction,
        )
    }

    resultStore.consumeResult<SearchLocationNavResult> { result ->
        when (val type = result.relayListType) {
            RelayListType.Single -> navigator.goBack(result = SelectLocationNavResult(true))
            is RelayListType.Multihop ->
                when (type.hopType) {
                    RelayHopType.ENTRY -> vm.selectRelayList(RelayHopType.EXIT)
                    RelayHopType.EXIT -> navigator.goBack(result = SelectLocationNavResult(true))
                }
        }
    }

    SelectLocationScreen(
        state = state.value,
        snackbarHostState = snackbarHostState,
        onSearchClick =
            dropUnlessResumed { relayListType ->
                navigator.navigate(SearchLocationNavKey(relayListType))
            },
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onFilterClick =
            dropUnlessResumed { filterTarget -> navigator.navigate(FilterNavKey(filterTarget)) },
        onFilterChipNavigate =
            dropUnlessResumed { filterChip -> navigator.navigateFromFilterChip(filterChip) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
        onSelectRelayList = vm::selectRelayList,
        onRecentsToggleEnableClick = vm::toggleRecentsEnabled,
        onRefreshRelayList = vm::refreshRelayList,
        onSetMultihopMode = vm::setMultihopMode,
        scrollToItem = vm::scrollToItem,
        relayListContent = { relayListType, bottomMargin ->
            RelayLists(
                relayListType = relayListType,
                bottomMargin = bottomMargin,
                onSelect = vm::selectSingle,
                onModifyMultihop = vm::modifyMultihop,
                onSelectAutomaticEntry = vm::selectAutomaticMultihopEntry,
                onAutomaticInfoClick =
                    dropUnlessResumed { navigator.navigate(AutomaticEntryInfoNavKey) },
                onAddCustomList =
                    dropUnlessResumed { navigator.navigate(CreateCustomListNavKey()) },
                onEditCustomLists = dropUnlessResumed { navigator.navigate(CustomListNavKey) },
                onUpdateBottomSheetState =
                    dropUnlessResumed { sheetState ->
                        navigator.navigate(LocationBottomSheetNavKey(sheetState))
                    },
                onSetMultihopToAlways = { vm.setMultihopMode(MultihopMode.ALWAYS) },
            )
        },
    )
}

@Suppress("LongMethod", "LongParameterList", "CyclomaticComplexMethod")
@Composable
fun SelectLocationScreen(
    state: Lc<Unit, SelectLocationUiState>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSearchClick: (RelayListType) -> Unit,
    onBackClick: () -> Unit,
    onFilterClick: (filterTarget: RelayHopType) -> Unit,
    onRecentsToggleEnableClick: () -> Unit,
    removeOwnershipFilter: (filterTarget: RelayHopType) -> Unit,
    removeProviderFilter: (filterTarget: RelayHopType) -> Unit,
    onFilterChipNavigate: (FilterChip) -> Unit,
    onSelectRelayList: (RelayHopType) -> Unit,
    onRefreshRelayList: () -> Unit,
    scrollToItem: (ScrollEvent) -> Unit,
    onSetMultihopMode: (MultihopMode) -> Unit,
    relayListContent: @Composable (RelayListType, bottomMargin: Dp) -> Unit,
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
                    imageVector = Icons.Rounded.Close,
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
                    modifier =
                        Modifier.testTag(SELECT_LOCATION_SEARCH_BUTTON_TEST_TAG)
                            .onGloballyPositioned { fabHeight = it.size.height },
                    onClick = { onSearchClick(state.value.relayListType) },
                    containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                    contentColor = MaterialTheme.colorScheme.onTertiary,
                ) {
                    Icon(
                        imageVector = Icons.Rounded.Search,
                        contentDescription = stringResource(id = R.string.search),
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
        },
        actions = {
            if (isTv()) {
                val isSearchButtonEnabled = state.contentOrNull()?.isSearchButtonEnabled == true
                SearchButton(
                    enabled = isSearchButtonEnabled,
                    onClick = { state.contentOrNull()?.let { onSearchClick(it.relayListType) } },
                )
            }
            val recentsCurrentlyEnabled = state.contentOrNull()?.isRecentsEnabled == true
            val activeMultihopMode =
                state.contentOrNull()?.activeMultihopMode ?: MultihopMode.WHEN_NEEDED
            val disabledText = stringResource(id = R.string.recents_disabled)
            val scope = rememberCoroutineScope()

            SelectLocationDropdownMenu(
                recentsEnabled = recentsCurrentlyEnabled,
                activeMultihopMode = activeMultihopMode,
                onRecentsToggleEnableClick = {
                    if (recentsCurrentlyEnabled) {
                        scope.launch { snackbarHostState.showSnackbarImmediately(disabledText) }
                    }
                    onRecentsToggleEnableClick()
                },
                onRefreshRelayList = onRefreshRelayList,
                onSetMultihopMode = onSetMultihopMode,
            )
        },
    ) { modifier ->
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
                        entryFilteringEnabled = state.value.isEntryFilteringEnabled,
                        entryFilterButtonEnabled = state.value.isEntryFilterButtonEnabled,
                        filterChips = state.value.filterChips,
                        hopSelection = state.value.hopSelection,
                        error = state.value.tunnelErrorStateCause,
                        userLocation = state.value.lastKnownLocation,
                        entryCountry = state.value.entryCountry,
                        hasAnyEntryFilter = state.value.hasAnyEntryFilter,
                        hasAnyExitFilter = state.value.hasAnyExitFilter,
                        onSelectRelayList = onSelectRelayList,
                        removeOwnershipFilter = {
                            removeOwnershipFilter(state.value.relayListType.hopType())
                        },
                        removeProviderFilter = {
                            removeProviderFilter(state.value.relayListType.hopType())
                        },
                        scrollToRelayItem = { relayListType: RelayListType, relayItem: RelayItem ->
                            scrollToItem(relayListType to relayItem)
                        },
                        onFilterClick = onFilterClick,
                        onFilterChipNavigate = onFilterChipNavigate,
                    )

                    relayListContent(state.value.relayListType, bottomMarginList)
                }
            }
        }
    }
}

@Composable
@Suppress("LongMethod")
private fun SelectLocationDropdownMenu(
    recentsEnabled: Boolean,
    activeMultihopMode: MultihopMode,
    onRecentsToggleEnableClick: () -> Unit,
    onRefreshRelayList: () -> Unit,
    onSetMultihopMode: (MultihopMode) -> Unit,
) {
    var showMenu by remember { mutableStateOf(false) }

    IconButton(
        modifier = Modifier.testTag(SELECT_LOCATION_MENU_BUTTON_TEST_TAG),
        onClick = { showMenu = !showMenu },
    ) {
        Icon(
            imageVector = Icons.Rounded.MoreVert,
            contentDescription = stringResource(R.string.more_actions),
        )
    }
    DropdownMenu(
        modifier = Modifier.background(MaterialTheme.colorScheme.tertiaryContainer),
        expanded = showMenu,
        onDismissRequest = { showMenu = false },
    ) {
        val colors =
            MenuDefaults.itemColors(
                leadingIconColor = MaterialTheme.colorScheme.onPrimary,
                disabledLeadingIconColor =
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
            )

        // Keep these assets in remember so we don't change them as we animate away the dropdown
        // menu
        var recentsItemTextId by remember {
            mutableIntStateOf(
                if (recentsEnabled) R.string.disable_recents else R.string.enable_recents
            )
        }
        var recentsIcon by remember {
            mutableStateOf(if (recentsEnabled) DeleteHistory else Icons.Rounded.History)
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
        DropdownMenuItem(
            text = { Text(text = stringResource(R.string.refresh_server_list)) },
            onClick = {
                showMenu = false
                onRefreshRelayList()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Rounded.Refresh, contentDescription = null) },
        )
        HorizontalDivider(
            modifier = Modifier.padding(top = Dimens.smallPadding, bottom = Dimens.mediumPadding),
            thickness = MenuDividerThickness,
        )
        Text(
            modifier = Modifier.padding(horizontal = Dimens.sideMarginNew),
            text = stringResource(R.string.multihop_mode),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(Dimens.tinyPadding))

        MultihopMode.WHEN_NEEDED.menuItem(activeMultihopMode).let { (icon, color) ->
            DropdownMenuItem(
                text = {
                    Text(
                        text = stringResource(R.string.when_needed),
                        color = color,
                    )
                },
                onClick = {
                    showMenu = false
                    onSetMultihopMode(MultihopMode.WHEN_NEEDED)
                },
                colors = colors.copy(leadingIconColor = color),
                leadingIcon = {
                    Icon(
                        icon,
                        contentDescription = null,
                    )
                },
            )
        }

        MultihopMode.ALWAYS.menuItem(activeMultihopMode).let { (icon, color) ->
            DropdownMenuItem(
                text = {
                    Text(
                        text = stringResource(R.string.always),
                        color = color,
                    )
                },
                onClick = {
                    showMenu = false
                    onSetMultihopMode(MultihopMode.ALWAYS)
                },
                colors = colors.copy(leadingIconColor = color),
                leadingIcon = {
                    Icon(
                        icon,
                        contentDescription = null,
                    )
                },
            )
        }

        MultihopMode.NEVER.menuItem(activeMultihopMode).let { (icon, color) ->
            DropdownMenuItem(
                text = {
                    Text(
                        text = stringResource(R.string.never),
                        color = color,
                    )
                },
                onClick = {
                    showMenu = false
                    onSetMultihopMode(MultihopMode.NEVER)
                },
                colors = colors.copy(leadingIconColor = color),
                leadingIcon = {
                    Icon(
                        icon,
                        contentDescription = null,
                    )
                },
            )
        }
    }
}

@Composable
private fun MultihopMode.menuItem(activeMultihopMode: MultihopMode): Pair<ImageVector, Color> {
    val positiveColor = MaterialTheme.colorScheme.positive
    val onPrimaryColor = MaterialTheme.colorScheme.onPrimary

    // This remember is used to "lock" the values of the icon and color to what they were when
    // we first show the menu, so that they won't change during the menu close animation.
    return remember(this) {
        val isSelected = this == activeMultihopMode

        val icon =
            if (isSelected) {
                Icons.Rounded.Check
            } else {
                when (this) {
                    MultihopMode.NEVER -> Icons.Outlined.WrongLocation
                    MultihopMode.ALWAYS -> Icons.Outlined.AddLocationAlt
                    MultihopMode.WHEN_NEEDED -> MultihopWhenNeeded
                }
            }

        val color = if (isSelected) positiveColor else onPrimaryColor

        icon to color
    }
}

@Composable
@Suppress("ComplexCondition")
private fun RelayLists(
    relayListType: RelayListType,
    bottomMargin: Dp,
    onSelect: (item: RelayItem) -> Unit,
    onModifyMultihop: (RelayItem, RelayHopType) -> Unit,
    onSetMultihopToAlways: () -> Unit,
    onSelectAutomaticEntry: () -> Unit,
    onAutomaticInfoClick: () -> Unit,
    onAddCustomList: () -> Unit,
    onEditCustomLists: (() -> Unit)?,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    val onSelectRelayItem: (RelayItem, RelayListType) -> Unit = { relayItem, relayListType ->
        if (relayListType is RelayListType.Multihop) {
            onModifyMultihop(relayItem, relayListType.hopType)
        } else {
            onSelect(relayItem)
        }
    }

    val lazyListStates = remember { mutableStateMapOf<RelayListType, LazyListState>() }

    Crossfade(relayListType) {
        when (it) {
            is RelayListType.Multihop ->
                when (it.hopType) {
                    RelayHopType.ENTRY ->
                        SelectLocationList(
                            relayListType = it,
                            bottomMargin = bottomMargin,
                            onSelectRelayItem = onSelectRelayItem,
                            onSetMultihopToAlways = onSetMultihopToAlways,
                            onSelectAutomaticEntry = onSelectAutomaticEntry,
                            onAutomaticInfoClick = onAutomaticInfoClick,
                            onAddCustomList = onAddCustomList,
                            onEditCustomLists = onEditCustomLists,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                            lazyListStates = lazyListStates,
                        )

                    RelayHopType.EXIT ->
                        SelectLocationList(
                            relayListType = it,
                            bottomMargin = bottomMargin,
                            onSelectRelayItem = onSelectRelayItem,
                            onSetMultihopToAlways = onSetMultihopToAlways,
                            onSelectAutomaticEntry = onSelectAutomaticEntry,
                            onAutomaticInfoClick = onAutomaticInfoClick,
                            onAddCustomList = onAddCustomList,
                            onEditCustomLists = onEditCustomLists,
                            onUpdateBottomSheetState = onUpdateBottomSheetState,
                            lazyListStates = lazyListStates,
                        )
                }

            RelayListType.Single ->
                SelectLocationList(
                    relayListType = it,
                    bottomMargin = bottomMargin,
                    onSelectRelayItem = onSelectRelayItem,
                    onSetMultihopToAlways = onSetMultihopToAlways,
                    onSelectAutomaticEntry = onSelectAutomaticEntry,
                    onAutomaticInfoClick = onAutomaticInfoClick,
                    onAddCustomList = onAddCustomList,
                    onEditCustomLists = onEditCustomLists,
                    onUpdateBottomSheetState = onUpdateBottomSheetState,
                    lazyListStates = lazyListStates,
                )
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Suppress("LongMethod", "CyclomaticComplexMethod")
@Composable
private fun SelectionContainer(
    progress: Float, // 0 - 1
    relayListType: RelayListType,
    hopSelection: HopSelection,
    error: ErrorStateCause?,
    userLocation: String?,
    entryCountry: String?,
    entryFilteringEnabled: Boolean,
    entryFilterButtonEnabled: Boolean,
    onFilterClick: (filterTarget: RelayHopType) -> Unit,
    hasAnyEntryFilter: Boolean,
    hasAnyExitFilter: Boolean,
    filterChips: List<FilterChip>,
    onSelectRelayList: (RelayHopType) -> Unit,
    removeOwnershipFilter: () -> Unit,
    removeProviderFilter: () -> Unit,
    onFilterChipNavigate: (FilterChip) -> Unit,
    scrollToRelayItem: (RelayListType, RelayItem) -> Unit,
) {

    var multihopListSelector by remember { mutableStateOf(RelayHopType.EXIT) }
    if (relayListType is RelayListType.Multihop) {
        multihopListSelector = relayListType.hopType
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
                        userLocation = userLocation,
                        exitLocation = hopSelection.relay.toDisplayName(),
                        errorText = error.errorText(RelayListType.Single),
                        expandProgress = progress,
                        filterState =
                            if (hasAnyExitFilter) FilterState.ActiveAndFiltersSelected
                            else FilterState.Active,
                        onFilterClick = { onFilterClick(RelayHopType.EXIT) },
                        onSelect = {
                            hopSelection.relay?.getOrNull()?.let {
                                scrollToRelayItem(RelayListType.Single, it)
                            }
                        },
                    )

                is HopSelection.Multi ->
                    MultihopSelector(
                        userLocation = userLocation,
                        exitSelected = multihopListSelector == RelayHopType.EXIT,
                        exitLocation = hopSelection.exit.toDisplayName(),
                        exitErrorText = error.errorText(RelayListType.Multihop(RelayHopType.EXIT)),
                        onExitClick = {
                            if (multihopListSelector == RelayHopType.EXIT) {
                                hopSelection.exit?.getOrNull()?.let {
                                    scrollToRelayItem(
                                        RelayListType.Multihop(RelayHopType.EXIT),
                                        it,
                                    )
                                }
                            } else {
                                onSelectRelayList(RelayHopType.EXIT)
                            }
                        },
                        entryLocation = hopSelection.entry.toDisplayName(entryCountry),
                        entryErrorText =
                            error.errorText(RelayListType.Multihop(RelayHopType.ENTRY)),
                        onEntryClick = {
                            if (multihopListSelector == RelayHopType.ENTRY) {
                                hopSelection.entry?.getOrNull()?.let {
                                    scrollToRelayItem(
                                        RelayListType.Multihop(RelayHopType.ENTRY),
                                        it,
                                    )
                                }
                            } else {
                                onSelectRelayList(RelayHopType.ENTRY)
                            }
                        },
                        entryFilterState =
                            when (entryFilteringEnabled to hasAnyEntryFilter) {
                                true to true -> FilterState.ActiveAndFiltersSelected
                                true to false -> FilterState.Active
                                false to true -> FilterState.InactiveAndFiltersSelected
                                else -> FilterState.Inactive
                            },
                        entryFilterButtonEnabled = entryFilterButtonEnabled,
                        exitFilterState =
                            if (hasAnyExitFilter) FilterState.ActiveAndFiltersSelected
                            else FilterState.Active,
                        onFilterClick = onFilterClick,
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
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            motionScene = scene,
            progress = progress,
        ) {
            FilterContainer(
                modifier = Modifier.layoutId(keyFilters).alpha(EaseInQuint.transform(progress)),
                filters = filterChips,
                relayFiltersActive =
                    entryFilteringEnabled || relayListType.hopType() == RelayHopType.EXIT,
                onRemoveOwnershipFilter = { removeOwnershipFilter() },
                onRemoveProviderFilter = { removeProviderFilter() },
                onFilterNavigate = onFilterChipNavigate,
            )
        }
    }
}

@Composable
fun Constraint<RelayItem>?.toDisplayName(entryCountry: String? = null) =
    when (this) {
        Constraint.Any ->
            if (entryCountry != null) stringResource(R.string.automatic_with_country, entryCountry)
            else stringResource(R.string.automatic)

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

        // This case happens if we are doing an auto multihop and the exit relay becomes invalid
        // e.g. because an incompatible owner or provider filter is set on the exit.
        ParameterGenerationError.NoMatchingRelayExit if relayListType is RelayListType.Single ->
            stringResource(R.string.no_matching_relay)

        ParameterGenerationError.NoMatchingRelayEntry if
            relayListType is RelayListType.Multihop && relayListType.hopType == RelayHopType.ENTRY
         -> stringResource(R.string.no_matching_relay)

        ParameterGenerationError.NoMatchingRelayExit if
            relayListType is RelayListType.Multihop && relayListType.hopType == RelayHopType.EXIT
         -> stringResource(R.string.no_matching_relay)

        else -> null
    }

private val MenuDividerThickness = 2.dp
