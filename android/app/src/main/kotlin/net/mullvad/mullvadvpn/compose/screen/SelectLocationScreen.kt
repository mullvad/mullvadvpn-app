package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.animateScrollBy
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.cell.NormalRelayLocationCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.ThreeDotCell
import net.mullvad.mullvadvpn.compose.communication.CreateCustomListDialogRequest
import net.mullvad.mullvadvpn.compose.communication.CreateCustomListDialogResult
import net.mullvad.mullvadvpn.compose.communication.CustomListLocationScreenRequest
import net.mullvad.mullvadvpn.compose.communication.EditCustomListNameDialogRequest
import net.mullvad.mullvadvpn.compose.communication.EditCustomListNameDialogResult
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.CreateCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListsDestination
import net.mullvad.mullvadvpn.compose.destinations.DeleteCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListNameDestination
import net.mullvad.mullvadvpn.compose.destinations.FilterScreenDestination
import net.mullvad.mullvadvpn.compose.extensions.showSnackbar
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.canAddLocation
import net.mullvad.mullvadvpn.viewmodel.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewSelectLocationScreen() {
    val state =
        SelectLocationUiState.Content(
            searchTerm = "",
            selectedOwnership = null,
            selectedProvidersCount = 0,
            countries = listOf(RelayItem.Country("Country 1", "Code 1", false, emptyList())),
            selectedItem = null,
            customLists = emptyList(),
            filteredCustomLists = emptyList()
        )
    AppTheme {
        SelectLocationScreen(
            state = state,
        )
    }
}

@Destination(style = SelectLocationTransition::class)
@Composable
fun SelectLocation(
    navigator: DestinationsNavigator,
    createCustomListDialogResultRecipient:
        ResultRecipient<CreateCustomListDestination, CreateCustomListDialogResult>,
    editCustomListNameDialogResultRecipient:
        ResultRecipient<EditCustomListNameDestination, EditCustomListNameDialogResult>,
    deleteCustomListDialogResultRecipent: ResultRecipient<DeleteCustomListDestination, String>,
) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle().value

    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    LaunchedEffectCollect(vm.uiSideEffect) {
            when (it) {
                SelectLocationSideEffect.CloseScreen -> navigator.navigateUp()
                is SelectLocationSideEffect.LocationAddedToCustomList -> {
                    launch {
                        snackbarHostState.currentSnackbarData?.dismiss()
                        snackbarHostState.showSnackbar(
                            message =
                                context.getString(
                                    R.string.country_was_added_to_list,
                                    it.item.name,
                                    it.customList.name
                                ),
                            actionLabel = context.getString(R.string.undo),
                            duration = SnackbarDuration.Long,
                            onAction = {
                                vm.removeLocationFromList(
                                    item = it.item,
                                    customList = it.customList
                                )
                            }
                        )
                    }
                }
            }
        }

    createCustomListDialogResultRecipient.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                when (
                    val dialogResult = result.value as? CreateCustomListDialogResult.WithLocation
                ) {
                    is CreateCustomListDialogResult.WithLocation -> {
                        scope.launch {
                            snackbarHostState.currentSnackbarData?.dismiss()
                            snackbarHostState.showSnackbar(
                                message =
                                    context.getString(
                                        R.string.country_was_added_to_list,
                                        dialogResult.locationName,
                                        dialogResult.customListName
                                    ),
                                actionLabel = context.getString(R.string.undo),
                                duration = SnackbarDuration.Long,
                                onAction = { vm.deleteCustomList(dialogResult.customListId) }
                            )
                        }
                    }
                }
            }
        }
    }

    editCustomListNameDialogResultRecipient.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                scope.launch {
                    snackbarHostState.currentSnackbarData?.dismiss()
                    snackbarHostState.showSnackbar(
                        message =
                            context.getString(R.string.name_was_changed_to, result.value.newName),
                        actionLabel = context.getString(R.string.undo),
                        duration = SnackbarDuration.Short,
                        onAction = {
                            vm.updateCustomListName(result.value.customListId, result.value.oldName)
                        }
                    )
                }
            }
        }
    }

    deleteCustomListDialogResultRecipent.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                scope.launch {
                    snackbarHostState.currentSnackbarData?.dismiss()
                    snackbarHostState.showSnackbar(
                        message =
                            context.getString(R.string.delete_custom_list_message, result.value),
                        actionLabel = context.getString(R.string.undo),
                        duration = SnackbarDuration.Long,
                        onAction = { vm.undoDeleteCustomList() }
                    )
                }
            }
        }
    }

    SelectLocationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = navigator::navigateUp,
<<<<<<< HEAD
        onFilterClick = { navigator.navigate(FilterScreenDestination, true) },
        onCreateCustomList = {
            navigator.navigate(CreateCustomListDestination) { launchSingleTop = true }
=======
        onFilterClick = { navigator.navigate(FilterScreenDestination) },
        onCreateCustomList = { relayItem ->
            navigator.navigate(
                CreateCustomListDestination(
                    if (relayItem != null) {
                        CreateCustomListDialogRequest.WithLocation(
                            locationCode = relayItem.code,
                            locationName = relayItem.name
                        )
                    } else {
                        CreateCustomListDialogRequest.FromScratch
                    }
                )
            ) {
                launchSingleTop = true
            }
>>>>>>> 896e56ffc (WIP)
        },
        onEditCustomLists = { navigator.navigate(CustomListsDestination()) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
        onAddLocationToList = vm::addLocationToList,
        onEditCustomListName = {
            navigator.navigate(
                EditCustomListNameDestination(
                    EditCustomListNameDialogRequest(customListId = it.id, name = it.name)
                )
            )
        },
        onEditLocationsCustomList = {
            navigator.navigate(
                CustomListLocationsDestination(
                    request =
                        CustomListLocationScreenRequest(customListKey = it.id, newList = false)
                )
            )
        },
        onDeleteCustomList = {
            navigator.navigate(DeleteCustomListDestination(id = it.id, name = it.name))
        }
    )
}

@Suppress("LongMethod")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    onCreateCustomList: (location: RelayItem?) -> Unit = {},
    onEditCustomLists: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
    onAddLocationToList: (location: RelayItem, customList: RelayItem.CustomList) -> Unit = { _, _ ->
    },
    onEditCustomListName: (RelayItem.CustomList) -> Unit = {},
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit = {},
    onDeleteCustomList: (RelayItem.CustomList) -> Unit = {}
) {
    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) }
            )
        }
    ) {
        Column(modifier = Modifier.padding(it).background(backgroundColor).fillMaxSize()) {
            Row(modifier = Modifier.fillMaxWidth()) {
                IconButton(onClick = onBackClick) {
                    Icon(
                        modifier = Modifier.rotate(270f),
                        painter = painterResource(id = R.drawable.icon_back),
                        tint = Color.Unspecified,
                        contentDescription = null,
                    )
                }
                Text(
                    text = stringResource(id = R.string.select_location),
                    modifier = Modifier.align(Alignment.CenterVertically).weight(weight = 1f),
                    textAlign = TextAlign.Center,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onPrimary,
                )
                IconButton(onClick = onFilterClick) {
                    Icon(
                        painter = painterResource(id = R.drawable.icons_more_circle),
                        contentDescription = null,
                        tint = Color.Unspecified,
                    )
                }
            }

            when (state) {
                SelectLocationUiState.Loading -> {}
                is SelectLocationUiState.Content -> {
                    if (state.hasFilter) {
                        FilterCell(
                            ownershipFilter = state.selectedOwnership,
                            selectedProviderFilter = state.selectedProvidersCount,
                            removeOwnershipFilter = removeOwnershipFilter,
                            removeProviderFilter = removeProviderFilter,
                        )
                    }
                }
            }

            SearchTextField(
                modifier =
                    Modifier.fillMaxWidth()
                        .height(Dimens.searchFieldHeight)
                        .padding(horizontal = Dimens.searchFieldHorizontalPadding),
                backgroundColor = MaterialTheme.colorScheme.tertiaryContainer,
                textColor = MaterialTheme.colorScheme.onTertiaryContainer,
            ) { searchString ->
                onSearchTermInput.invoke(searchString)
            }
            Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
            val lazyListState = rememberLazyListState()
            if (
                state is SelectLocationUiState.Content &&
                    state.inSearch.not() &&
                    state.selectedItem != null
            ) {
                LaunchedEffect(state.selectedItem) {
                    val index = state.indexOfSelectedRelayItem()

                    if (index >= 0) {
                        lazyListState.scrollToItem(index)
                        lazyListState.animateScrollAndCentralizeItem(index)
                    }
                }
            }
            var bottomSheetState by remember {
                mutableStateOf<BottomSheetState>(BottomSheetState.Hidden)
            }
            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                when (state) {
                    SelectLocationUiState.Loading -> {
                        loading()
                    }
                    is SelectLocationUiState.Content -> {
                        if (state.showCustomLists) {
                            customLists(
                                customLists = state.filteredCustomLists,
                                selectedItem = state.selectedItem,
                                onSelectRelay = onSelectRelay,
                                onShowCustomListBottomSheet = {
                                    bottomSheetState =
                                        BottomSheetState.ShowCustomListsBottomSheet(
                                            state.customLists.isNotEmpty()
                                        )
                                },
                                onShowEditBottomSheet = { customList ->
                                    bottomSheetState =
                                        BottomSheetState.ShowEditCustomListBottomSheet(customList)
                                }
                            )
                            item { Spacer(modifier = Modifier.height(Dimens.mediumPadding)) }
                        }
                        if (state.countries.isNotEmpty()) {
                            relayList(
                                countries = state.countries,
                                selectedItem = state.selectedItem,
                                onSelectRelay = onSelectRelay,
                                onShowLocationBottomSheet = { location ->
                                    bottomSheetState =
                                        BottomSheetState.ShowLocationBottomSheet(
                                            customLists = uiState.customLists,
                                            item = location
                                        )
                                }
                            )
                        } else {
                            item { LocationsEmptyText(searchTerm = state.searchTerm) }
                        }
                    }
                }
            when (bottomSheetState) {
                is BottomSheetState.ShowCustomListsBottomSheet -> {
                    CustomListsBottomSheet(
                        bottomSheetState =
                            bottomSheetState as BottomSheetState.ShowCustomListsBottomSheet,
                        onCreateCustomList = { onCreateCustomList(null) },
                        onEditCustomLists = onEditCustomLists,
                        closeBottomSheet = { bottomSheetState = BottomSheetState.Hidden }
                    )
                }
                is BottomSheetState.ShowLocationBottomSheet -> {
                    LocationBottomSheet(
                        customLists =
                            (bottomSheetState as BottomSheetState.ShowLocationBottomSheet)
                                .customLists,
                        item = (bottomSheetState as BottomSheetState.ShowLocationBottomSheet).item,
                        onCreateCustomList = onCreateCustomList,
                        onAddLocationToList = onAddLocationToList,
                        closeBottomSheet = { bottomSheetState = BottomSheetState.Hidden }
                    )
                }
                is BottomSheetState.ShowEditCustomListBottomSheet -> {
                    EditCustomListBottomSheet(
                        customList =
                            (bottomSheetState as BottomSheetState.ShowEditCustomListBottomSheet)
                                .customList,
                        onEditName = onEditCustomListName,
                        onEditLocations = onEditLocationsCustomList,
                        onDeleteCustomList = onDeleteCustomList,
                        closeBottomSheet = { bottomSheetState = BottomSheetState.Hidden }
                    )
                }
                BottomSheetState.Hidden -> {
                    /* Do nothing */
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

private fun LazyListScope.customLists(
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItem?,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowCustomListBottomSheet: () -> Unit,
    onShowEditBottomSheet: (RelayItem.CustomList) -> Unit
) {
    item(
        contentType = { ContentType.HEADER },
    ) {
        ThreeDotCell(
            text = stringResource(R.string.custom_lists),
            onClickDots = onShowCustomListBottomSheet,
        )
    }
    if (customLists.isNotEmpty()) {
        items(
            items = customLists,
            key = { item -> item.hashCode() },
            contentType = { ContentType.ITEM },
        ) { customList ->
            NormalRelayLocationCell(
                relay = customList,
                // Do not show selection for locations in custom lists
                selectedItem = selectedItem as? RelayItem.CustomList,
                onSelectRelay = onSelectRelay,
                onLongClick = {
                    if (it is RelayItem.CustomList) {
                        onShowEditBottomSheet(it)
                    }
                },
                modifier = Modifier.animateContentSize(),
            )
        }
        item {
            SwitchComposeSubtitleCell(text = stringResource(R.string.to_add_locations_to_a_list))
        }
    } else {
        item(contentType = ContentType.EMPTY_TEXT) {
            SwitchComposeSubtitleCell(text = stringResource(R.string.to_create_a_custom_list))
        }
    }
}

private fun LazyListScope.relayList(
    countries: List<RelayItem.Country>,
    selectedItem: RelayItem?,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowLocationBottomSheet: (item: RelayItem) -> Unit,
) {
    item(
        contentType = ContentType.HEADER,
    ) {
        HeaderCell(
            text = stringResource(R.string.all_locations),
        )
    }
    items(
        items = countries,
        key = { item -> item.hashCode() },
        contentType = { ContentType.ITEM },
    ) { country ->
        NormalRelayLocationCell(
            relay = country,
            selectedItem = selectedItem,
            onSelectRelay = onSelectRelay,
            onLongClick = onShowLocationBottomSheet,
            modifier = Modifier.animateContentSize(),
        )
    }
}

private fun SelectLocationUiState.Content.indexOfSelectedRelayItem(): Int {
    val rawIndex =
        countries.indexOfFirst { relayCountry ->
            relayCountry.location.location.country ==
                when (selectedItem) {
                    is RelayItem.Country -> selectedItem.code
                    is RelayItem.City -> selectedItem.location.countryCode
                    is RelayItem.Relay -> selectedItem.location.countryCode
                    is RelayItem.CustomList,
                    null -> null
                }
        }
    return if (rawIndex >= 0) {
        // Extra items are: Custom list header, custom list description and locations header
        return rawIndex + customLists.size + EXTRA_ITEMS
    } else {
        rawIndex
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CustomListsBottomSheet(
    bottomSheetState: BottomSheetState.ShowCustomListsBottomSheet,
    onCreateCustomList: () -> Unit,
    onEditCustomLists: () -> Unit,
    closeBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    ModalBottomSheet(
        onDismissRequest = closeBottomSheet,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.background,
        dragHandle = {
            BottomSheetDefaults.DragHandle(color = MaterialTheme.colorScheme.onBackground)
        }
    ) {
        HeaderCell(
            text = stringResource(id = R.string.edit_custom_lists),
            background = MaterialTheme.colorScheme.background
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.new_list),
            onClick = {
                onCreateCustomList()
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor = MaterialTheme.colorScheme.onBackground
        )
        IconCell(
            iconId = R.drawable.icon_edit,
            title = stringResource(id = R.string.edit_lists),
            onClick = {
                onEditCustomLists()
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor =
                MaterialTheme.colorScheme.onBackground.copy(
                    alpha =
                        if (bottomSheetState.editListEnabled) {
                            AlphaVisible
                        } else {
                            AlphaInactive
                        }
                ),
            enabled = bottomSheetState.editListEnabled
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LocationBottomSheet(
    customLists: List<RelayItem.CustomList>,
    item: RelayItem,
    onCreateCustomList: (relayItem: RelayItem) -> Unit,
    onAddLocationToList: (location: RelayItem, customList: RelayItem.CustomList) -> Unit,
    closeBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    ModalBottomSheet(
        onDismissRequest = closeBottomSheet,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.background,
        dragHandle = {
            BottomSheetDefaults.DragHandle(color = MaterialTheme.colorScheme.onBackground)
        }
    ) {
        HeaderCell(
            text = stringResource(id = R.string.add_location_to_list, item.name),
            background = MaterialTheme.colorScheme.background
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
        customLists.forEach {
            val enabled = it.canAddLocation(item)
            IconCell(
                background = MaterialTheme.colorScheme.background,
                titleColor =
                    if (enabled) {
                        MaterialTheme.colorScheme.onBackground
                    } else {
                        MaterialTheme.colorScheme.onSecondary
                    },
                iconId = null,
                title =
                    if (enabled) {
                        it.name
                    } else {
                        stringResource(id = R.string.location_added, it.name)
                    },
                onClick = {
                    onAddLocationToList(item, it)
                    scope
                        .launch { sheetState.hide() }
                        .invokeOnCompletion {
                            if (!sheetState.isVisible) {
                                closeBottomSheet()
                            }
                        }
                },
                enabled = enabled
            )
        }
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.new_list),
            onClick = {
                onCreateCustomList(item)
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor = MaterialTheme.colorScheme.onBackground
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditCustomListBottomSheet(
    customList: RelayItem.CustomList,
    onEditName: (item: RelayItem.CustomList) -> Unit,
    onEditLocations: (item: RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (item: RelayItem.CustomList) -> Unit,
    closeBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    ModalBottomSheet(
        onDismissRequest = closeBottomSheet,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.background,
        dragHandle = {
            BottomSheetDefaults.DragHandle(color = MaterialTheme.colorScheme.onBackground)
        }
    ) {
        HeaderCell(
            text = stringResource(id = R.string.edit_custom_lists),
            background = MaterialTheme.colorScheme.background
        )
        IconCell(
            iconId = R.drawable.icon_edit,
            title = stringResource(id = R.string.edit_name),
            onClick = {
                onEditName(customList)
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor = MaterialTheme.colorScheme.onBackground
        )
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.edit_locations),
            onClick = {
                onEditLocations(customList)
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor = MaterialTheme.colorScheme.onBackground
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
        IconCell(
            iconId = R.drawable.icon_delete,
            title = stringResource(id = R.string.delete),
            onClick = {
                onDeleteCustomList(customList)
                scope
                    .launch { sheetState.hide() }
                    .invokeOnCompletion {
                        if (!sheetState.isVisible) {
                            closeBottomSheet()
                        }
                    }
            },
            background = MaterialTheme.colorScheme.background,
            titleColor = MaterialTheme.colorScheme.onBackground
        )
    }
}

suspend fun LazyListState.animateScrollAndCentralizeItem(index: Int) {
    val itemInfo = this.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }
    if (itemInfo != null) {
        val center = layoutInfo.viewportEndOffset / 2
        val childCenter = itemInfo.offset + itemInfo.size / 2
        animateScrollBy((childCenter - center).toFloat())
    } else {
        animateScrollToItem(index)
    }
}

private const val EXTRA_ITEMS = 3

sealed interface BottomSheetState {
    data object Hidden : BottomSheetState

    data class ShowCustomListsBottomSheet(val editListEnabled: Boolean) : BottomSheetState

    data class ShowLocationBottomSheet(
        val customLists: List<RelayItem.CustomList>,
        val item: RelayItem
    ) : BottomSheetState

    data class ShowEditCustomListBottomSheet(val customList: RelayItem.CustomList) :
        BottomSheetState
}
