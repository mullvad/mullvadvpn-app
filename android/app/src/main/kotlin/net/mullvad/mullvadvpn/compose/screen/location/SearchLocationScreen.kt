package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SearchSelectLocationUiState
import net.mullvad.mullvadvpn.compose.transitions.SearchTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationSideEffect
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewSearchLocationScreen() {
    AppTheme { SearchLocationScreen(state = SearchSelectLocationUiState.NoQuery("", emptyList())) }
}

data class SearchLocationNavArgs(val relayListSelection: RelayListSelection)

@Composable
@Destination<RootGraph>(style = SearchTransition::class, navArgs = SearchLocationNavArgs::class)
fun SearchLocation(
    navigator: DestinationsNavigator,
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
) {
    val viewModel = koinViewModel<SearchLocationViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            SearchLocationSideEffect.CloseScreen ->
                navigator.popBackStack(route = ConnectDestination, inclusive = false)
            is SearchLocationSideEffect.CustomListActionToast ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.resultData,
                        onUndo = viewModel::performAction,
                    )
                }
            SearchLocationSideEffect.GenericError ->
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
        viewModel::performAction,
    )

    editCustomListNameDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        viewModel::performAction,
    )

    deleteCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        viewModel::performAction,
    )

    updateCustomListResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        viewModel::performAction,
    )

    SearchLocationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSelectRelay = viewModel::selectRelay,
        onToggleExpand = viewModel::onToggleExpand,
        onSearchInputChanged = viewModel::onSearchInputUpdated,
        onEditCustomLists = dropUnlessResumed { navigator.navigate(CustomListsDestination()) },
        onAddLocationToList = viewModel::addLocationToList,
        onRemoveLocationFromList = viewModel::removeLocationFromList,
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
        onRemoveOwnershipFilter = viewModel::removeOwnerFilter,
        onRemoveProviderFilter = viewModel::removeProviderFilter,
        onGoBack = dropUnlessResumed { navigator.navigateUp() },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchLocationScreen(
    state: SearchSelectLocationUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onSelectRelay: (RelayItem) -> Unit = {},
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit = { _, _, _ -> },
    onSearchInputChanged: (String) -> Unit = {},
    onCreateCustomList: (location: RelayItem.Location?) -> Unit = {},
    onEditCustomLists: () -> Unit = {},
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit =
        { _, _ ->
        },
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit =
        { _, _ ->
        },
    onEditCustomListName: (RelayItem.CustomList) -> Unit = {},
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit = {},
    onDeleteCustomList: (RelayItem.CustomList) -> Unit = {},
    onRemoveOwnershipFilter: () -> Unit = {},
    onRemoveProviderFilter: () -> Unit = {},
    onGoBack: () -> Unit = {},
) {
    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        }
    ) {
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

        val backgroundColor = MaterialTheme.colorScheme.surface
        val onBackgroundColor = MaterialTheme.colorScheme.onSurface
        val keyboardController = LocalSoftwareKeyboardController.current
        SearchBar(
            inputField = {
                SearchBarDefaults.InputField(
                    query = state.searchTerm,
                    onQueryChange = onSearchInputChanged,
                    onSearch = { keyboardController?.hide() },
                    expanded = true,
                    onExpandedChange = {},
                    leadingIcon = {
                        IconButton(onClick = onGoBack) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Default.ArrowBack,
                                contentDescription = stringResource(R.string.back),
                            )
                        }
                    },
                    trailingIcon = {
                        if (state.searchTerm.isNotEmpty()) {
                            IconButton(onClick = { onSearchInputChanged("") }) {
                                Icon(
                                    imageVector = Icons.Default.Close,
                                    contentDescription = stringResource(R.string.clear_input),
                                )
                            }
                        }
                    },
                    placeholder = { Text(text = stringResource(id = R.string.search_placeholder)) },
                    colors =
                        TextFieldDefaults.colors(
                            focusedContainerColor = backgroundColor,
                            unfocusedContainerColor = backgroundColor,
                            focusedPlaceholderColor = onBackgroundColor,
                            unfocusedPlaceholderColor = onBackgroundColor,
                            focusedTextColor = onBackgroundColor,
                            unfocusedTextColor = onBackgroundColor,
                            cursorColor = onBackgroundColor,
                            focusedLeadingIconColor = onBackgroundColor,
                            unfocusedLeadingIconColor = onBackgroundColor,
                            focusedTrailingIconColor = onBackgroundColor,
                            unfocusedTrailingIconColor = onBackgroundColor,
                        ),
                )
            },
            expanded = true,
            onExpandedChange = {},
            colors =
                SearchBarDefaults.colors(
                    containerColor = backgroundColor,
                    dividerColor = MaterialTheme.colorScheme.onSurface,
                ),
            modifier = Modifier.padding(it),
        ) {
            val lazyListState = rememberLazyListState()
            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .background(color = backgroundColor)
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                when (state) {
                    is SearchSelectLocationUiState.NoQuery -> {
                        noQuery()
                        filterRow(
                            filters = state.filterChips,
                            onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                            onRemoveProviderFilter = onRemoveProviderFilter,
                        )
                    }
                    is SearchSelectLocationUiState.Content -> {
                        filterRow(
                            filters = state.filterChips,
                            onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                            onRemoveProviderFilter = onRemoveProviderFilter,
                        )
                        relayListContent(
                            backgroundColor = backgroundColor,
                            customLists = state.customLists,
                            relayListItems = state.relayListItems,
                            relayListSelection = state.relayListSelection,
                            onSelectRelay = onSelectRelay,
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = { newSheetState ->
                                locationBottomSheetState = newSheetState
                            },
                        )
                    }
                }
            }
        }
    }
}

private fun LazyListScope.noQuery() {
    item(contentType = ContentType.DESCRIPTION) {
        Text(
            text = "Type at least 2 characters to start searching",
            style = MaterialTheme.typography.labelMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(Dimens.mediumPadding),
        )
    }
}

private fun LazyListScope.filterRow(
    filters: List<FilterChip>,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
) {
    if (filters.isNotEmpty()) {
        item {
            FilterRow(
                filters = filters,
                onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                onRemoveProviderFilter = onRemoveProviderFilter,
            )
        }
    }
}
