package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.text.HtmlCompat
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterCell
import net.mullvad.mullvadvpn.compose.cell.RelayLocationCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

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
            uiCloseAction = MutableSharedFlow(),
            enterTransitionEndAction = MutableSharedFlow()
        )
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun SelectLocationScreen(
    uiState: SelectLocationUiState,
    uiCloseAction: SharedFlow<Unit>,
    enterTransitionEndAction: SharedFlow<Unit>,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {}
) {

    val backgroundColor = MaterialTheme.colorScheme.background
    val systemUiController = rememberSystemUiController()

    LaunchedEffect(Unit) { uiCloseAction.collect { onBackClick() } }
    LaunchedEffect(Unit) {
        enterTransitionEndAction.collect { systemUiController.setStatusBarColor(backgroundColor) }
    }

    val (backFocus, listFocus, searchBarFocus) = remember { FocusRequester.createRefs() }
    Column(modifier = Modifier.background(backgroundColor).fillMaxWidth().fillMaxHeight()) {
        Row(
            modifier =
                Modifier.padding(vertical = Dimens.selectLocationTitlePadding)
                    .padding(end = Dimens.selectLocationTitlePadding)
                    .fillMaxWidth()
        ) {
            IconButton(onClick = onBackClick) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_back),
                    contentDescription = null,
                    tint = Color.Unspecified,
                    modifier = Modifier.size(Dimens.titleIconSize).rotate(270f)
                )
            }
            Text(
                text = stringResource(id = R.string.select_location),
                modifier =
                    Modifier.align(Alignment.CenterVertically)
                        .weight(weight = 1f)
                        .padding(end = Dimens.titleIconSize),
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onPrimary
            )
            IconButton(onClick = onFilterClick) {
                Icon(
                    painter = painterResource(id = R.drawable.icons_more_circle),
                    contentDescription = null,
                    tint = Color.Unspecified,
                    modifier = Modifier.size(Dimens.titleIconSize)
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
                    .focusRequester(searchBarFocus)
                    .focusProperties { next = backFocus }
                    .height(Dimens.searchFieldHeight)
                    .padding(horizontal = Dimens.searchFieldHorizontalPadding)
        ) { searchString ->
            onSearchTermInput.invoke(searchString)
        }
        Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
        val lazyListState = rememberLazyListState()
        LazyColumn(
            modifier =
                Modifier.focusRequester(listFocus)
                    .fillMaxSize()
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
                                textResource(id = R.string.select_location_empty_text_second_row)
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
