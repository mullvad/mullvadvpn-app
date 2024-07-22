package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.ExperimentalFoundationApi
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
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SheetState
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
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
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.generated.destinations.FilterDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec
import kotlinx.coroutines.launch
import mullvad_daemon.management_interface.customList
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.cell.RelayItemCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.ThreeDotCell
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListSuccess
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.RunOnKeyChange
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
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
            relayListItems = emptyList(),
            customLists = emptyList(),
            selectedItem = null
        )
    AppTheme {
        SelectLocationScreen(
            state = state,
        )
    }
}

@Destination<RootGraph>(style = SelectLocationTransition::class)
@Suppress("LongMethod")
@Composable
fun SelectLocation(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<Boolean>,
    createCustomListDialogResultRecipient: ResultRecipient<CreateCustomListDestination, Created>,
    editCustomListNameDialogResultRecipient:
        ResultRecipient<EditCustomListNameDestination, Renamed>,
    deleteCustomListDialogResultRecipient: ResultRecipient<DeleteCustomListDestination, Deleted>,
    updateCustomListResultRecipient:
        ResultRecipient<CustomListLocationsDestination, LocationsChanged>
) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle().value

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            SelectLocationSideEffect.CloseScreen -> backNavigator.navigateBack(result = true)
            is SelectLocationSideEffect.LocationAddedToCustomList ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.result,
                        onUndo = vm::performAction
                    )
                }
            is SelectLocationSideEffect.LocationRemovedFromCustomList ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.result,
                        onUndo = vm::performAction
                    )
                }
            SelectLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.error_occurred),
                        duration = SnackbarDuration.Short
                    )
                }
        }
    }

    createCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    editCustomListNameDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    deleteCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    updateCustomListResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    SelectLocationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() },
        onFilterClick = dropUnlessResumed { navigator.navigate(FilterDestination) },
        onCreateCustomList =
            dropUnlessResumed { relayItem ->
                navigator.navigate(
                    CreateCustomListDestination(locationCode = relayItem?.id),
                )
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
                        initialName = customList.customList.name
                    ),
                )
            },
        onEditLocationsCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigate(
                    CustomListLocationsDestination(customListId = customList.id, newList = false),
                )
            },
        onDeleteCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigate(
                    DeleteCustomListDestination(
                        customListId = customList.id,
                        name = customList.customList.name
                    ),
                )
            }
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Suppress("LongMethod")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    onCreateCustomList: (location: RelayItem.Location?) -> Unit = {},
    onEditCustomLists: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit =
        { _, _ ->
        },
    onRemoveLocationFromList: (location: RelayItem.Location, customList: CustomListId) -> Unit =
        { _, _ ->
        },
    onEditCustomListName: (RelayItem.CustomList) -> Unit = {},
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit = {},
    onDeleteCustomList: (RelayItem.CustomList) -> Unit = {},
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit = { _, _, _ -> },
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
        var bottomSheetState by remember { mutableStateOf<BottomSheetState?>(null) }
        BottomSheets(
            bottomSheetState = bottomSheetState,
            onCreateCustomList = onCreateCustomList,
            onEditCustomLists = onEditCustomLists,
            onAddLocationToList = onAddLocationToList,
            onRemoveLocationFromList = onRemoveLocationFromList,
            onEditCustomListName = onEditCustomListName,
            onEditLocationsCustomList = onEditLocationsCustomList,
            onDeleteCustomList = onDeleteCustomList,
            onHideBottomSheet = { bottomSheetState = null }
        )

        Column(modifier = Modifier.padding(it).background(backgroundColor).fillMaxSize()) {
            SelectLocationTopBar(onBackClick = onBackClick, onFilterClick = onFilterClick)

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
            val selectedItemCode = (state as? SelectLocationUiState.Content)?.selectedItem ?: ""
            RunOnKeyChange(key = selectedItemCode) {
                val index = state.indexOfSelectedRelayItem()

                if (index >= 0) {
                    lazyListState.scrollToItem(index)
                    lazyListState.animateScrollAndCentralizeItem(index)
                }
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
                        items(state.relayListItems, key = { a -> a.key }, contentType = { null }) {
                            when (it) {
                                RelayListItem.CustomListHeader ->
                                    CustomListHeader(
                                        onShowCustomListBottomSheet = {
                                            bottomSheetState =
                                                BottomSheetState.ShowCustomListsBottomSheet(
                                                    editListEnabled = false
                                                )
                                        }
                                    )
                                is RelayListItem.CustomListItem ->
                                    CustomListItem(
                                        it,
                                        onSelectRelay,
                                        {
                                            bottomSheetState =
                                                BottomSheetState.ShowEditCustomListBottomSheet(it)
                                        },
                                        { customListId, expand ->
                                            onToggleExpand(customListId, null, expand)
                                        }
                                    )
                                is RelayListItem.CustomListEntryItem ->
                                    CustomListEntryItem(
                                        it,
                                        { onSelectRelay(it.item) },
                                        {
                                            bottomSheetState =
                                                BottomSheetState.ShowCustomListsEntryBottomSheet(
                                                    it.parentId,
                                                    it.item
                                                )
                                        },
                                        { expand: Boolean ->
                                            onToggleExpand(it.item.id, it.parentId, expand)
                                        }
                                    )
                                is RelayListItem.CustomListFooter -> CustomListFooter(it)
                                RelayListItem.LocationHeader -> RelayLocationHeader()
                                is RelayListItem.GeoLocationItem ->
                                    RelayLocationItem(
                                        it,
                                        { onSelectRelay(it.item) },
                                        {
                                            bottomSheetState =
                                                BottomSheetState.ShowLocationBottomSheet(
                                                    state.customLists,
                                                    it.item
                                                )
                                        },
                                        { expand -> onToggleExpand(it.item.id, null, expand) }
                                    )
                                is RelayListItem.LocationsEmptyText ->
                                    LocationsEmptyText(it.searchTerm)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun LazyItemScope.RelayLocationHeader() {
    HeaderCell(text = stringResource(R.string.all_locations), modifier = Modifier.animateItem())
}

@Composable
fun LazyItemScope.RelayLocationItem(
    relayItem: RelayListItem.GeoLocationItem,
    onSelectRelay: () -> Unit,
    onLongClick: () -> Unit,
    onExpand: (Boolean) -> Unit,
) {
    val location = relayItem.item
    RelayItemCell(
        location,
        relayItem.isSelected,
        { onSelectRelay() },
        { onLongClick() },
        { onExpand(it) },
        relayItem.expanded,
        relayItem.depth,
        modifier = Modifier.animateItem()
    )
}

@Composable
fun LazyItemScope.CustomListItem(
    itemState: RelayListItem.CustomListItem,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowEditBottomSheet: (RelayItem.CustomList) -> Unit,
    onExpand: ((CustomListId, Boolean) -> Unit),
) {
    val customListItem = itemState.item
    RelayItemCell(
        customListItem,
        itemState.isSelected,
        { onSelectRelay(customListItem) },
        { onShowEditBottomSheet(customListItem) },
        { onExpand(customListItem.id, it) },
        itemState.expanded,
        0,
        modifier = Modifier.animateItem()
    )
}

@Composable
fun LazyItemScope.CustomListEntryItem(
    itemState: RelayListItem.CustomListEntryItem,
    onSelectRelay: () -> Unit,
    onShowEditCustomListEntryBottomSheet: () -> Unit,
    onToggleExpand: (Boolean) -> Unit,
) {
    val customListEntryItem = itemState.item
    RelayItemCell(
        customListEntryItem,
        false,
        onSelectRelay,
        onShowEditCustomListEntryBottomSheet,
        onToggleExpand,
        itemState.expanded,
        itemState.depth,
        modifier = Modifier.animateItem()
    )
}

@Composable
fun LazyItemScope.CustomListFooter(item: RelayListItem.CustomListFooter) {
    SwitchComposeSubtitleCell(
        text =
            if (item.hasCustomList) {
                stringResource(R.string.to_add_locations_to_a_list)
            } else {
                stringResource(R.string.to_create_a_custom_list)
            },
        modifier = Modifier.background(MaterialTheme.colorScheme.background).animateItem()
    )
}

@Composable
private fun SelectLocationTopBar(onBackClick: () -> Unit, onFilterClick: () -> Unit) {
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
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR))
    }
}

@Composable
private fun LazyItemScope.CustomListHeader(onShowCustomListBottomSheet: () -> Unit) {
    ThreeDotCell(
        text = stringResource(R.string.custom_lists),
        onClickDots = onShowCustomListBottomSheet,
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG).animateItem()
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun BottomSheets(
    bottomSheetState: BottomSheetState?,
    onCreateCustomList: (RelayItem.Location?) -> Unit,
    onEditCustomLists: () -> Unit,
    onAddLocationToList: (RelayItem.Location, RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, parent: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onHideBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    val onCloseBottomSheet: (animate: Boolean) -> Unit = { animate ->
        if (animate) {
            scope
                .launch { sheetState.hide() }
                .invokeOnCompletion {
                    if (!sheetState.isVisible) {
                        onHideBottomSheet()
                    }
                }
        } else {
            onHideBottomSheet()
        }
    }
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface

    when (bottomSheetState) {
        is BottomSheetState.ShowCustomListsBottomSheet -> {
            CustomListsBottomSheet(
                sheetState = sheetState,
                onBackgroundColor = onBackgroundColor,
                bottomSheetState = bottomSheetState,
                onCreateCustomList = { onCreateCustomList(null) },
                onEditCustomLists = onEditCustomLists,
                closeBottomSheet = onCloseBottomSheet
            )
        }
        is BottomSheetState.ShowLocationBottomSheet -> {
            LocationBottomSheet(
                sheetState = sheetState,
                onBackgroundColor = onBackgroundColor,
                customLists = bottomSheetState.customLists,
                item = bottomSheetState.item,
                onCreateCustomList = onCreateCustomList,
                onAddLocationToList = onAddLocationToList,
                closeBottomSheet = onCloseBottomSheet
            )
        }
        is BottomSheetState.ShowEditCustomListBottomSheet -> {
            EditCustomListBottomSheet(
                sheetState = sheetState,
                onBackgroundColor = onBackgroundColor,
                customList = bottomSheetState.customList,
                onEditName = onEditCustomListName,
                onEditLocations = onEditLocationsCustomList,
                onDeleteCustomList = onDeleteCustomList,
                closeBottomSheet = onCloseBottomSheet
            )
        }
        is BottomSheetState.ShowCustomListsEntryBottomSheet -> {
            CustomListEntryBottomSheet(
                sheetState = sheetState,
                onBackgroundColor = onBackgroundColor,
                customListId = bottomSheetState.parentId,
                item = bottomSheetState.item,
                onRemoveLocationFromList = onRemoveLocationFromList,
                closeBottomSheet = onCloseBottomSheet
            )
        }
        null -> {
            /* Do nothing */
        }
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CustomListsBottomSheet(
    onBackgroundColor: Color,
    sheetState: SheetState,
    bottomSheetState: BottomSheetState.ShowCustomListsBottomSheet,
    onCreateCustomList: () -> Unit,
    onEditCustomLists: () -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG)
    ) {
        HeaderCell(
            text = stringResource(id = R.string.edit_custom_lists),
            background = Color.Unspecified
        )
        HorizontalDivider(color = onBackgroundColor)
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.new_list),
            titleColor = onBackgroundColor,
            onClick = {
                onCreateCustomList()
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
        IconCell(
            iconId = R.drawable.icon_edit,
            title = stringResource(id = R.string.edit_lists),
            titleColor =
                onBackgroundColor.copy(
                    alpha =
                        if (bottomSheetState.editListEnabled) {
                            AlphaVisible
                        } else {
                            AlphaInactive
                        }
                ),
            onClick = {
                onEditCustomLists()
                closeBottomSheet(true)
            },
            background = Color.Unspecified,
            enabled = bottomSheetState.editListEnabled
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LocationBottomSheet(
    onBackgroundColor: Color,
    sheetState: SheetState,
    customLists: List<RelayItem.CustomList>,
    item: RelayItem.Location,
    onCreateCustomList: (relayItem: RelayItem.Location) -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG)
    ) { ->
        HeaderCell(
            text = stringResource(id = R.string.add_location_to_list, item.name),
            background = Color.Unspecified
        )
        HorizontalDivider(color = onBackgroundColor)
        customLists.forEach {
            val enabled = it.canAddLocation(item)
            IconCell(
                iconId = null,
                title =
                    if (enabled) {
                        it.name
                    } else {
                        stringResource(id = R.string.location_added, it.name)
                    },
                titleColor =
                    if (enabled) {
                        onBackgroundColor
                    } else {
                        MaterialTheme.colorScheme.onSecondary
                    },
                onClick = {
                    onAddLocationToList(item, it)
                    closeBottomSheet(true)
                },
                background = Color.Unspecified,
                enabled = enabled
            )
        }
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.new_list),
            titleColor = onBackgroundColor,
            onClick = {
                onCreateCustomList(item)
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditCustomListBottomSheet(
    onBackgroundColor: Color,
    sheetState: SheetState,
    customList: RelayItem.CustomList,
    onEditName: (item: RelayItem.CustomList) -> Unit,
    onEditLocations: (item: RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (item: RelayItem.CustomList) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        onDismissRequest = { closeBottomSheet(false) }
    ) {
        HeaderCell(text = customList.name, background = Color.Unspecified)
        IconCell(
            iconId = R.drawable.icon_edit,
            title = stringResource(id = R.string.edit_name),
            titleColor = onBackgroundColor,
            onClick = {
                onEditName(customList)
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.edit_locations),
            titleColor = onBackgroundColor,
            onClick = {
                onEditLocations(customList)
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
        HorizontalDivider(color = onBackgroundColor)
        IconCell(
            iconId = R.drawable.icon_delete,
            title = stringResource(id = R.string.delete),
            titleColor = onBackgroundColor,
            onClick = {
                onDeleteCustomList(customList)
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CustomListEntryBottomSheet(
    onBackgroundColor: Color,
    sheetState: SheetState,
    customListId: CustomListId,
    item: RelayItem.Location,
    onRemoveLocationFromList: (location: RelayItem.Location, customList: CustomListId) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG)
    ) {
        HeaderCell(
            text = stringResource(id = R.string.remove_location_from_list, item.name),
            background = Color.Unspecified
        )
        HorizontalDivider(color = onBackgroundColor)

        IconCell(
            iconId = R.drawable.ic_remove,
            title = stringResource(id = R.string.remove_button),
            titleColor = onBackgroundColor,
            onClick = {
                onRemoveLocationFromList(item, customListId)
                closeBottomSheet(true)
            },
            background = Color.Unspecified
        )
    }
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

private suspend fun SnackbarHostState.showResultSnackbar(
    context: Context,
    result: CustomListSuccess,
    onUndo: (CustomListAction) -> Unit
) {
    showSnackbarImmediately(
        message = result.message(context),
        actionLabel = context.getString(R.string.undo),
        duration = SnackbarDuration.Long,
        onAction = { onUndo(result.undo) }
    )
}

private fun CustomListSuccess.message(context: Context): String =
    when (this) {
        is Created ->
            locationNames.firstOrNull()?.let { locationName ->
                context.getString(R.string.location_was_added_to_list, locationName, name)
            } ?: context.getString(R.string.locations_were_changed_for, name)
        is Deleted -> context.getString(R.string.delete_custom_list_message, name)
        is Renamed -> context.getString(R.string.name_was_changed_to, name)
        is LocationsChanged -> context.getString(R.string.locations_were_changed_for, name)
    }

@Composable
private fun <D : DestinationSpec, R : CustomListSuccess> ResultRecipient<D, R>
    .OnCustomListNavResult(
    snackbarHostState: SnackbarHostState,
    performAction: (action: CustomListAction) -> Unit
) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    this.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                // Handle result
                scope.launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = result.value,
                        onUndo = performAction
                    )
                }
            }
        }
    }
}

sealed interface BottomSheetState {

    data class ShowCustomListsBottomSheet(val editListEnabled: Boolean) : BottomSheetState

    data class ShowCustomListsEntryBottomSheet(
        val parentId: CustomListId,
        val item: RelayItem.Location
    ) : BottomSheetState

    data class ShowLocationBottomSheet(
        val customLists: List<RelayItem.CustomList>,
        val item: RelayItem.Location
    ) : BottomSheetState

    data class ShowEditCustomListBottomSheet(val customList: RelayItem.CustomList) :
        BottomSheetState
}
