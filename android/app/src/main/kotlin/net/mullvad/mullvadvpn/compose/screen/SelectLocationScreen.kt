package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.text.HtmlCompat
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.RelayLocationCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.FilterScreenDestination
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewSelectLocationScreen() {
    val state =
        SelectLocationUiState.ShowData(
            searchTerm = "",
            countries = listOf(RelayCountry("Country 1", "Code 1", false, emptyList())),
            selectedRelay = null,
            selectedOwnership = null,
            selectedProvidersCount = 0
        )
    AppTheme {
        SelectLocationScreen(
            uiState = state,
        )
    }
}

@Destination(style = SelectLocationTransition::class)
@Composable
fun SelectLocation(navigator: DestinationsNavigator) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsState().value
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect {
            when (it) {
                SelectLocationSideEffect.CloseScreen -> navigator.navigateUp()
            }
        }
    }

    SelectLocationScreen(
        uiState = state,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = navigator::navigateUp,
        onFilterClick = { navigator.navigate(FilterScreenDestination) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter
    )
}

@Composable
fun SelectLocationScreen(
    uiState: SelectLocationUiState,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {}
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
                    color = MaterialTheme.colorScheme.onPrimary
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
                is SelectLocationUiState.ShowData -> {
                    if (uiState.hasFilter) {
                        FilterCell(
                            ownershipFilter = uiState.selectedOwnership,
                            selectedProviderFilter = uiState.selectedProvidersCount,
                            removeOwnershipFilter = removeOwnershipFilter,
                            removeProviderFilter = removeProviderFilter
                        )
                    }
                }
            }

            SearchTextField(
                modifier =
                    Modifier.fillMaxWidth()
                        .height(Dimens.searchFieldHeight)
                        .padding(horizontal = Dimens.searchFieldHorizontalPadding)
            ) { searchString ->
                onSearchTermInput.invoke(searchString)
            }
            Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
            val lazyListState = rememberLazyListState()
            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar)
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                when (uiState) {
                    SelectLocationUiState.Loading -> {
                        item(contentType = ContentType.PROGRESS) {
                            MullvadCircularProgressIndicatorLarge(
                                Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR)
                            )
                        }
                    }
                    is SelectLocationUiState.ShowData -> {
                        if (uiState.countries.isEmpty()) {
                            item(contentType = ContentType.EMPTY_TEXT) {
                                val firstRow =
                                    HtmlCompat.fromHtml(
                                            textResource(
                                                id = R.string.select_location_empty_text_first_row,
                                                uiState.searchTerm
                                            ),
                                            HtmlCompat.FROM_HTML_MODE_COMPACT
                                        )
                                        .toAnnotatedString(boldFontWeight = FontWeight.ExtraBold)
                                val secondRow =
                                    textResource(
                                        id = R.string.select_location_empty_text_second_row
                                    )
                                Column(
                                    modifier =
                                        Modifier.padding(
                                            horizontal = Dimens.selectLocationTitlePadding
                                        ),
                                    horizontalAlignment = Alignment.CenterHorizontally
                                ) {
                                    Text(
                                        text = firstRow,
                                        style = MaterialTheme.typography.labelMedium,
                                        textAlign = TextAlign.Center,
                                        color = MaterialTheme.colorScheme.onSecondary,
                                        maxLines = 2,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                    Text(
                                        text = secondRow,
                                        style = MaterialTheme.typography.labelMedium,
                                        textAlign = TextAlign.Center,
                                        color = MaterialTheme.colorScheme.onSecondary
                                    )
                                }
                            }
                        } else {
                            items(
                                count = uiState.countries.size,
                                key = { index -> uiState.countries[index].hashCode() },
                                contentType = { ContentType.ITEM }
                            ) { index ->
                                val country = uiState.countries[index]
                                RelayLocationCell(
                                    relay = country,
                                    selectedItem = uiState.selectedRelay,
                                    onSelectRelay = onSelectRelay,
                                    modifier = Modifier.animateContentSize()
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
