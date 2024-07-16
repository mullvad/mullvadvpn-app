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
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
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
import com.ramcosta.composedestinations.generated.destinations.CustomListEntrySheetDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListSheetDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsSheetDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.generated.destinations.FilterScreenDestination
import com.ramcosta.composedestinations.generated.destinations.LocationSheetDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.StatusRelayLocationCell
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
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.RunOnKeyChange
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
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
            countries =
                listOf(
                    RelayItem.Location.Country(
                        GeoLocationId.Country("Country 1"), "Code 1", false, emptyList())),
            selectedItem = null,
            customLists = emptyList(),
            filteredCustomLists = emptyList())
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
                        context = context, result = it.result, onUndo = vm::performAction)
                }
            is SelectLocationSideEffect.LocationRemovedFromCustomList ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context, result = it.result, onUndo = vm::performAction)
                }
            SelectLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.error_occurred),
                        duration = SnackbarDuration.Short)
                }
        }
    }

    createCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState, vm::performAction)

    editCustomListNameDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState, vm::performAction)

    deleteCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState, vm::performAction)

    updateCustomListResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    SelectLocationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() },
        onFilterClick = dropUnlessResumed { navigator.navigate(FilterScreenDestination) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
        showCustomListBottomSheet =
            dropUnlessResumed { navigator.navigate(CustomListsSheetDestination(true)) },
        showLocationBottomSheet =
            dropUnlessResumed { name, location ->
                navigator.navigate(LocationSheetDestination(name, location))
            },
        showEditCustomListBottomSheet =
            dropUnlessResumed { customListId: CustomListId, customListName: CustomListName ->
                navigator.navigate(CustomListSheetDestination(customListId, customListName))
            },
        showEditCustomListEntryBottomSheet =
            dropUnlessResumed {
                locationName: String,
                customList: CustomListId,
                location: GeoLocationId ->
                navigator.navigate(
                    CustomListEntrySheetDestination(locationName, customList, location))
            },
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
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
    showCustomListBottomSheet: () -> Unit = {},
    showEditCustomListBottomSheet: (CustomListId, CustomListName) -> Unit = { _, _ -> },
    showEditCustomListEntryBottomSheet: (String, CustomListId, GeoLocationId) -> Unit = { _, _, _ ->
    },
    showLocationBottomSheet: (String, GeoLocationId) -> Unit = { _, _ -> },
) {
    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) })
        }) {
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
                            if (state.showCustomLists) {
                                customLists(
                                    customLists = state.filteredCustomLists,
                                    selectedItem = state.selectedItem,
                                    backgroundColor = backgroundColor,
                                    onSelectRelay = onSelectRelay,
                                    onShowCustomListBottomSheet = showCustomListBottomSheet,
                                    onShowEditBottomSheet = showEditCustomListBottomSheet,
                                    onShowEditCustomListEntryBottomSheet = {
                                        item: RelayItem.Location,
                                        customList: RelayItem.CustomList ->
                                        showEditCustomListEntryBottomSheet(
                                            item.name, customList.id, item.id)
                                    })
                                item {
                                    Spacer(
                                        modifier =
                                            Modifier.height(Dimens.mediumPadding).animateItem())
                                }
                            }
                            if (state.countries.isNotEmpty()) {
                                relayList(
                                    countries = state.countries,
                                    selectedItem = state.selectedItem,
                                    onSelectRelay = onSelectRelay,
                                    onShowLocationBottomSheet = { location ->
                                        showLocationBottomSheet(location.name, location.id)
                                    })
                            }
                            if (state.showEmpty) {
                                item { LocationsEmptyText(searchTerm = state.searchTerm) }
                            }
                        }
                    }
                }
            }
        }
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

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.customLists(
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItemId?,
    backgroundColor: Color,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowCustomListBottomSheet: () -> Unit,
    onShowEditBottomSheet: (CustomListId, CustomListName) -> Unit,
    onShowEditCustomListEntryBottomSheet: (item: RelayItem.Location, RelayItem.CustomList) -> Unit
) {
    item(
        contentType = { ContentType.HEADER },
    ) {
        ThreeDotCell(
            text = stringResource(R.string.custom_lists),
            onClickDots = onShowCustomListBottomSheet,
            modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG).animateItem())
    }
    if (customLists.isNotEmpty()) {
        items(
            items = customLists,
            key = { item -> item.id },
            contentType = { ContentType.ITEM },
        ) { customList ->
            StatusRelayLocationCell(
                relay = customList,
                // Do not show selection for locations in custom lists
                selectedItem = selectedItem as? CustomListId,
                onSelectRelay = onSelectRelay,
                onLongClick = {
                    if (it is RelayItem.CustomList) {
                        onShowEditBottomSheet(it.id, it.customListName)
                    } else if (it is RelayItem.Location && it in customList.locations) {
                        onShowEditCustomListEntryBottomSheet(it, customList)
                    }
                },
                modifier = Modifier.animateItem(),
            )
        }
        item {
            SwitchComposeSubtitleCell(
                text = stringResource(R.string.to_add_locations_to_a_list),
                modifier = Modifier.background(backgroundColor).animateItem())
        }
    } else {
        item(contentType = ContentType.EMPTY_TEXT) {
            SwitchComposeSubtitleCell(
                text = stringResource(R.string.to_create_a_custom_list),
                modifier = Modifier.background(backgroundColor).animateItem())
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.relayList(
    countries: List<RelayItem.Location.Country>,
    selectedItem: RelayItemId?,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowLocationBottomSheet: (item: RelayItem.Location) -> Unit,
) {
    item(
        key = "all_locations_header",
        contentType = ContentType.HEADER,
    ) {
        HeaderCell(text = stringResource(R.string.all_locations), modifier = Modifier.animateItem())
    }
    items(
        items = countries,
        key = { item -> item.id },
        contentType = { ContentType.ITEM },
    ) { country ->
        StatusRelayLocationCell(
            relay = country,
            selectedItem = selectedItem,
            onSelectRelay = onSelectRelay,
            onLongClick = { onShowLocationBottomSheet(it as RelayItem.Location) },
            modifier = Modifier.animateItem())
    }
}

private fun SelectLocationUiState.indexOfSelectedRelayItem(): Int =
    if (this is SelectLocationUiState.Content) {
        when (selectedItem) {
            is CustomListId ->
                filteredCustomLists.indexOfFirst { it.id == selectedItem } + EXTRA_ITEM_CUSTOM_LIST
            is GeoLocationId ->
                countries.indexOfFirst { it.id == selectedItem.country } +
                    customLists.size +
                    EXTRA_ITEMS_LOCATION
            else -> -1
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

private suspend fun SnackbarHostState.showResultSnackbar(
    context: Context,
    result: CustomListSuccess,
    onUndo: (CustomListAction) -> Unit
) {
    showSnackbarImmediately(
        message = result.message(context),
        actionLabel = context.getString(R.string.undo),
        duration = SnackbarDuration.Long,
        onAction = { onUndo(result.undo) })
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
                        context = context, result = result.value, onUndo = performAction)
                }
            }
        }
    }
}

private const val EXTRA_ITEMS_LOCATION =
    4 // Custom lists header, custom lists description, spacer, all locations header
private const val EXTRA_ITEM_CUSTOM_LIST = 1 // Custom lists header
