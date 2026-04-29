package net.mullvad.mullvadvpn.feature.location.impl.search

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CreateCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.DeleteCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.UpdateCustomListNavResult
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetNavKey
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.api.SearchLocationNavResult
import net.mullvad.mullvadvpn.feature.location.impl.ContentType
import net.mullvad.mullvadvpn.feature.location.impl.EmptyRelayListText
import net.mullvad.mullvadvpn.feature.location.impl.FilterRow
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.showResultSnackbar
import net.mullvad.mullvadvpn.feature.location.impl.relayListContent
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.MullvadSearchBar
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.usecase.FilterChip
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loading|Default|No Locations|Not found|Results")
@Composable
private fun PreviewSearchLocationScreen(
    @PreviewParameter(SearchLocationsUiStatePreviewParameterProvider::class)
    state: Lce<Unit, SearchLocationUiState, Unit>
) {
    AppTheme {
        SearchLocationScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onSelectRelayItem = { _, _ -> },
            onToggleExpand = { _, _, _ -> },
            onSearchInputChanged = {},
            onRemoveOwnershipFilter = {},
            onRemoveProviderFilter = {},
            navigateToBottomSheet = {},
            onGoBack = {},
        )
    }
}

@Suppress("LongMethod", "CyclomaticComplexMethod")
@Composable
fun SearchLocation(relayListType: RelayListType, navigator: Navigator) {

    val viewModel = koinViewModel<SearchLocationViewModel> { parametersOf(relayListType) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    val resultStore = LocalResultStore.current

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is SearchLocationSideEffect.LocationSelected ->
                navigator.goBack(result = SearchLocationNavResult(it.relayListType))

            SearchLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.error_occurred)
                    )
                }
            is SearchLocationSideEffect.EntryAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(
                                R.string.relay_item_already_selected_as_entry,
                                it.relayItem.name,
                            )
                    )
                }
            is SearchLocationSideEffect.ExitAlreadySelected ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(
                                R.string.relay_item_already_selected_as_exit,
                                it.relayItem.name,
                            )
                    )
                }
            is SearchLocationSideEffect.RelayItemInactive -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(R.string.relayitem_is_inactive, it.relayItem.name)
                    )
                }
            }
        }
    }

    resultStore.consumeResult<CreateCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = viewModel::performAction,
        )
    }

    resultStore.consumeResult<EditCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = viewModel::performAction,
        )
    }

    resultStore.consumeResult<DeleteCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = viewModel::performAction,
        )
    }

    resultStore.consumeResult<UpdateCustomListNavResult> { result ->
        snackbarHostState.showResultSnackbar(
            resources = resources,
            result = result.value,
            onUndo = viewModel::performAction,
        )
    }

    SearchLocationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSelectRelayItem = viewModel::selectRelayItem,
        onToggleExpand = viewModel::onToggleExpand,
        onSearchInputChanged = viewModel::onSearchInputUpdated,
        onRemoveOwnershipFilter = viewModel::removeOwnerFilter,
        onRemoveProviderFilter = viewModel::removeProviderFilter,
        navigateToBottomSheet =
            dropUnlessResumed { sheetState ->
                navigator.navigate(LocationBottomSheetNavKey(sheetState))
            },
        onGoBack = dropUnlessResumed { navigator.goBack() },
    )
}

@Suppress("LongMethod", "LongParameterList")
@Composable
fun SearchLocationScreen(
    state: Lce<Unit, SearchLocationUiState, Unit>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelayItem: (RelayItem, RelayListType) -> Unit,
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit,
    onSearchInputChanged: (String) -> Unit,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
    onGoBack: () -> Unit,
    navigateToBottomSheet: (LocationBottomSheetState) -> Unit,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        }
    ) {
        Column(modifier = Modifier.padding(it)) {
            val focusRequester = remember { FocusRequester() }
            LaunchedEffect(state is Lce.Content) { focusRequester.requestFocus() }
            MullvadSearchBar(
                modifier = Modifier.focusRequester(focusRequester),
                searchTerm = state.contentOrNull()?.searchTerm ?: "",
                enabled = state is Lce.Content,
                onSearchInputChanged = onSearchInputChanged,
                hideKeyboard = { keyboardController?.hide() },
                onGoBack = onGoBack,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.onSurface)
            val lazyListState = rememberLazyListState()
            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .padding(horizontal = Dimens.mediumPadding)
                        .background(color = MaterialTheme.colorScheme.surface)
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                if (state is Lce.Content) {
                    filterRow(
                        filters = state.value.filterChips,
                        onRemoveOwnershipFilter = onRemoveOwnershipFilter,
                        onRemoveProviderFilter = onRemoveProviderFilter,
                    )
                }
                when (state) {
                    is Lce.Loading -> {
                        loading()
                    }
                    is Lce.Error -> {
                        // Relay list is empty
                        item { EmptyRelayListText() }
                    }
                    is Lce.Content -> {
                        relayListContent(
                            relayListItems = state.value.relayListItems,
                            relayListType = state.value.relayListType,
                            onSelectRelayItem = {
                                onSelectRelayItem(it, state.value.relayListType)
                            },
                            onToggleExpand = onToggleExpand,
                            onUpdateBottomSheetState = navigateToBottomSheet,
                            customListHeader = {
                                ListHeader(
                                    content = {
                                        Text(
                                            text = stringResource(R.string.custom_lists),
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                    }
                                )
                            },
                            locationHeader = {
                                ListHeader(
                                    content = {
                                        Text(
                                            text = stringResource(R.string.locations),
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                    }
                                )
                            },
                        )
                    }
                }
            }
        }
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

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) { MullvadCircularProgressIndicatorLarge() }
}
