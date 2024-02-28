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
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.CreateCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListsDestination
import net.mullvad.mullvadvpn.compose.destinations.FilterScreenDestination
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.relaylist.RelayItem
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
            customLists = emptyList()
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
    createCustomListResultRecipient: ResultRecipient<CreateCustomListDestination, String>
) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect {
            when (it) {
                SelectLocationSideEffect.CloseScreen -> navigator.navigateUp()
            }
        }
    }

    createCustomListResultRecipient.onNavResult(
        listener = { result ->
            when (result) {
                NavResult.Canceled -> {
                    /* Do nothing */
                }
                is NavResult.Value -> {
                    navigator.navigate(
                        CustomListLocationsDestination(customListKey = result.value, newList = true)
                    )
                }
            }
        }
    )

    SelectLocationScreen(
        state = state,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = navigator::navigateUp,
        onFilterClick = { navigator.navigate(FilterScreenDestination) },
        onCreateCustomList = {
            navigator.navigate(CreateCustomListDestination) { launchSingleTop = true }
        },
        onEditCustomLists = { navigator.navigate(CustomListsDestination()) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
    )
}

@Suppress("LongMethod")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    onCreateCustomList: () -> Unit = {},
    onEditCustomLists: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
) {
    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold {
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
            var showBottomSheet by remember { mutableStateOf(false) }
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
                                customLists = state.customLists,
                                selectedItem = state.selectedItem,
                                onSelectRelay = onSelectRelay,
                                onShowBottomSheet = { showBottomSheet = true },
                            )
                            item { Spacer(modifier = Modifier.height(Dimens.mediumPadding)) }
                        }
                        if (state.countries.isNotEmpty()) {
                            relayList(
                                countries = state.countries,
                                selectedItem = state.selectedItem,
                                onSelectRelay = onSelectRelay,
                            )
                        } else {
                            item { LocationsEmptyText(searchTerm = state.searchTerm) }
                        }
                    }
                }
            }
            CustomListsBottomSheet(
                showBottomSheet = showBottomSheet,
                editListEnabled =
                    state is SelectLocationUiState.Content && state.customLists.isNotEmpty(),
                onCreateCustomList = onCreateCustomList,
                onEditCustomLists = onEditCustomLists,
                closeBottomSheet = { showBottomSheet = false }
            )
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
    onShowBottomSheet: () -> Unit,
) {
    item(
        contentType = { ContentType.HEADER },
    ) {
        ThreeDotCell(
            text = stringResource(R.string.custom_lists),
            onClickDots = onShowBottomSheet,
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
    onSelectRelay: (item: RelayItem) -> Unit
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
            modifier = Modifier.animateContentSize(),
        )
    }
}

private fun SelectLocationUiState.Content.indexOfSelectedRelayItem(): Int =
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CustomListsBottomSheet(
    showBottomSheet: Boolean,
    editListEnabled: Boolean,
    onCreateCustomList: () -> Unit,
    onEditCustomLists: () -> Unit,
    closeBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    if (showBottomSheet) {
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
                titleColor = MaterialTheme.colorScheme.onBackground,
                enabled = editListEnabled
            )
        }
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
