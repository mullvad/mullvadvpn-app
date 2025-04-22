package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
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
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.SearchLocationsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Default|No Locations|Not found|Results")
@Composable
private fun PreviewSearchLocationScreen(
    @PreviewParameter(SearchLocationsUiStatePreviewParameterProvider::class)
    state: SearchLocationUiState
) {
    AppTheme {
        SearchLocationScreen(
            state = state,
            SnackbarHostState(),
            {},
            { _, _, _ -> },
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
            {},
        )
    }
}

data class SearchLocationNavArgs(val relayListType: RelayListType)

@Suppress("LongMethod")
@Composable
@Destination<RootGraph>(style = TopLevelTransition::class, navArgs = SearchLocationNavArgs::class)
fun SearchLocation(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<RelayListType>,
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
            is SearchLocationSideEffect.LocationSelected ->
                backNavigator.navigateBack(result = it.relayListType)
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
                        message = context.getString(R.string.error_occurred)
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
        onCreateCustomList =
            dropUnlessResumed { relayItem ->
                navigator.navigate(CreateCustomListDestination(locationCode = relayItem?.id))
            },
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

@Suppress("LongMethod")
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchLocationScreen(
    state: SearchLocationUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (RelayItem) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onSearchInputChanged: (String) -> Unit,
    onCreateCustomList: (location: RelayItem.Location?) -> Unit,
    onEditCustomLists: () -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
    onGoBack: () -> Unit,
) {
    val backgroundColor = MaterialTheme.colorScheme.surface
    val onBackgroundColor = MaterialTheme.colorScheme.onSurface
    val keyboardController = LocalSoftwareKeyboardController.current
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
        Column(modifier = Modifier.padding(it)) {
            val focusRequester = remember { FocusRequester() }
            LaunchedEffect(Unit) { focusRequester.requestFocus() }
            SearchBar(
                modifier = Modifier.focusRequester(focusRequester),
                searchTerm = state.searchTerm,
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                onSearchInputChanged = onSearchInputChanged,
                hideKeyboard = { keyboardController?.hide() },
                onGoBack = onGoBack,
            )
            HorizontalDivider(color = onBackgroundColor)
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
                filterRow(
                    filters = state.filterChips,
                    onBackgroundColor = onBackgroundColor,
                    onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                    onRemoveProviderFilter = onRemoveProviderFilter,
                )
                when (state) {
                    is SearchLocationUiState.Content -> {
                        relayListContent(
                            backgroundColor = backgroundColor,
                            customLists = state.customLists,
                            relayListItems = state.relayListItems,
                            onSelectRelay = onSelectRelay,
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = { newSheetState ->
                                locationBottomSheetState = newSheetState
                            },
                            customListHeader = {
                                Title(
                                    text = stringResource(R.string.custom_lists),
                                    onBackgroundColor = onBackgroundColor,
                                )
                            },
                            locationHeader = {
                                Title(
                                    text = stringResource(R.string.locations),
                                    onBackgroundColor = onBackgroundColor,
                                )
                            },
                        )
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SearchBar(
    searchTerm: String,
    backgroundColor: Color,
    onBackgroundColor: Color,
    onSearchInputChanged: (String) -> Unit,
    hideKeyboard: () -> Unit,
    onGoBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    SearchBarDefaults.InputField(
        modifier = modifier.height(Dimens.searchFieldHeightExpanded).fillMaxWidth(),
        query = searchTerm,
        onQueryChange = onSearchInputChanged,
        onSearch = { hideKeyboard() },
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
            if (searchTerm.isNotEmpty()) {
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
}

private fun LazyListScope.filterRow(
    filters: List<FilterChip>,
    onBackgroundColor: Color,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
) {
    if (filters.isNotEmpty()) {
        item {
            Title(text = stringResource(R.string.filters), onBackgroundColor = onBackgroundColor)
        }
        item {
            FilterRow(
                filters = filters,
                showTitle = false,
                onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                onRemoveProviderFilter = onRemoveProviderFilter,
            )
        }
    }
}

@Composable
private fun Title(text: String, onBackgroundColor: Color) {
    Text(
        text = text,
        color = onBackgroundColor,
        modifier =
            Modifier.fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.smallPadding),
        style = MaterialTheme.typography.labelMedium,
    )
}
