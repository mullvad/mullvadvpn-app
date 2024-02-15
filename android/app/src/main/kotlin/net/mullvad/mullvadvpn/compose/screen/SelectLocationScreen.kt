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
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults.itemColors
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
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
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.CustomListCell
import net.mullvad.mullvadvpn.compose.cell.DropdownMenuCell
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.NormalRelayLocationCell
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.CreateCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.FilterScreenDestination
import net.mullvad.mullvadvpn.compose.state.RelayListState
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
        SelectLocationUiState.Data(
            searchTerm = "",
            selectedOwnership = null,
            selectedProvidersCount = 0,
            relayListState =
                RelayListState.RelayList(
                    countries =
                        listOf(RelayItem.Country("Country 1", "Code 1", false, emptyList())),
                    selectedItem = null,
                    customLists = emptyList(),
                ),
        )
    AppTheme {
        SelectLocationScreen(
            uiState = state,
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
    val state = vm.uiState.collectAsState().value
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
        uiState = state,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = navigator::navigateUp,
        onFilterClick = { navigator.navigate(FilterScreenDestination) },
        onCreateCustomList = {
            navigator.navigate(CreateCustomListDestination) { launchSingleTop = true }
        },
        onEditCustomList = { navigator.navigate(EditCustomListDestination(it.id)) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
    )
}

@Composable
fun SelectLocationScreen(
    uiState: SelectLocationUiState,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    onCreateCustomList: () -> Unit = {},
    onEditCustomList: (customList: RelayItem.CustomList) -> Unit = {},
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

            when (uiState) {
                SelectLocationUiState.Loading -> {}
                is SelectLocationUiState.Data -> {
                    if (uiState.hasFilter) {
                        FilterCell(
                            ownershipFilter = uiState.selectedOwnership,
                            selectedProviderFilter = uiState.selectedProvidersCount,
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
                uiState is SelectLocationUiState.Data &&
                    uiState.relayListState is RelayListState.RelayList &&
                    uiState.relayListState.selectedItem != null
            ) {
                LaunchedEffect(uiState.relayListState.selectedItem) {
                    val index = uiState.relayListState.indexOfSelectedRelayItem()

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
                when (uiState) {
                    SelectLocationUiState.Loading -> {
                        loading()
                    }
                    is SelectLocationUiState.Data -> {
                        if (uiState.relayListState is RelayListState.RelayList) {
                            customLists(
                                relayListState = uiState.relayListState,
                                onCreateCustomList = onCreateCustomList,
                                onEditCustomList = { showBottomSheet = true },
                                onSelectRelay = onSelectRelay,
                            )
                        }
                        item { Spacer(modifier = Modifier.height(Dimens.mediumPadding)) }
                        relayList(
                            relayListState = uiState.relayListState,
                            searchTerm = uiState.searchTerm,
                            onSelectRelay = onSelectRelay,
                        )
                    }
                }
            }
            EditListsBottomSheet(
                showBottomSheet = showBottomSheet,
                uiState = uiState,
                onCustomListClicked = onEditCustomList,
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
    relayListState: RelayListState.RelayList,
    onCreateCustomList: () -> Unit,
    onEditCustomList: () -> Unit,
    onSelectRelay: (item: RelayItem) -> Unit
) {
    item(
        contentType = { ContentType.HEADER },
    ) {
        DropdownMenuCell(
            text = stringResource(R.string.custom_lists),
            contextMenuItems = { onClose ->
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.create_new)) },
                    leadingIcon = {
                        Icon(
                            painter = painterResource(id = R.drawable.icon_add),
                            contentDescription = null,
                        )
                    },
                    colors =
                        itemColors()
                            .copy(
                                leadingIconColor = MaterialTheme.colorScheme.onBackground,
                                textColor = MaterialTheme.colorScheme.onBackground,
                            ),
                    onClick = {
                        onCreateCustomList()
                        onClose()
                    }
                )
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.edit_lists)) },
                    leadingIcon = {
                        Icon(
                            painter = painterResource(id = R.drawable.icon_edit),
                            contentDescription = null,
                        )
                    },
                    colors =
                        itemColors()
                            .copy(
                                leadingIconColor = MaterialTheme.colorScheme.onBackground,
                                textColor = MaterialTheme.colorScheme.onBackground,
                            ),
                    enabled = relayListState.customLists.isNotEmpty(),
                    onClick = {
                        onEditCustomList()
                        onClose()
                    }
                )
            }
        )
    }
    items(
        count = relayListState.customLists.size,
        key = { index -> relayListState.customLists[index].hashCode() },
        contentType = { ContentType.ITEM },
    ) { index ->
        val customList = relayListState.customLists[index]
        NormalRelayLocationCell(
            relay = customList,
            // Do not show selection for locations in custom lists
            selectedItem = relayListState.selectedItem as? RelayItem.CustomList,
            onSelectRelay = onSelectRelay,
            modifier = Modifier.animateContentSize(),
        )
    }
}

private fun LazyListScope.relayList(
    relayListState: RelayListState,
    searchTerm: String,
    onSelectRelay: (item: RelayItem) -> Unit
) {
    item(
        contentType = { ContentType.HEADER },
    ) {
        HeaderCell(
            text = stringResource(R.string.all_locations),
        )
    }
    when (relayListState) {
        is RelayListState.RelayList -> {
            items(
                count = relayListState.countries.size,
                key = { index -> relayListState.countries[index].hashCode() },
                contentType = { ContentType.ITEM },
            ) { index ->
                val country = relayListState.countries[index]
                NormalRelayLocationCell(
                    relay = country,
                    selectedItem = relayListState.selectedItem,
                    onSelectRelay = onSelectRelay,
                    modifier = Modifier.animateContentSize(),
                )
            }
        }
        RelayListState.Empty -> {
            if (searchTerm.isNotEmpty())
                item(contentType = ContentType.EMPTY_TEXT, key = CommonContentKey.EMPTY) {
                    LocationsEmptyText(searchTerm = searchTerm)
                }
        }
    }
}

private fun RelayListState.RelayList.indexOfSelectedRelayItem(): Int =
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
private fun EditListsBottomSheet(
    showBottomSheet: Boolean,
    uiState: SelectLocationUiState,
    onCustomListClicked: (RelayItem.CustomList) -> Unit,
    closeBottomSheet: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState()
    val scope = rememberCoroutineScope()

    if (
        showBottomSheet &&
            uiState is SelectLocationUiState.Data &&
            uiState.relayListState is RelayListState.RelayList
    ) {
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
            uiState.relayListState.customLists.forEach { customList ->
                CustomListCell(
                    customList = customList,
                    onCellClicked = {
                        onCustomListClicked(it)
                        scope
                            .launch { sheetState.hide() }
                            .invokeOnCompletion {
                                if (!sheetState.isVisible) {
                                    closeBottomSheet()
                                }
                            }
                    }
                )
                Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
            }
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
