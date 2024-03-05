package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.CheckableRelayLocationCell
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.DiscardChangesDialogDestination
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsSideEffect
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview
fun PreviewCustomListLocationScreen() {
    AppTheme { CustomListLocationsScreen(uiState = CustomListLocationsUiState.Content.Data()) }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun CustomListLocations(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListResult.LocationsChanged>,
    discardChangesResultRecipient: ResultRecipient<DiscardChangesDialogDestination, Boolean>,
    customListId: String,
    newList: Boolean,
) {
    val customListsViewModel =
        koinViewModel<CustomListLocationsViewModel>(
            parameters = { parametersOf(customListId, newList) }
        )

    discardChangesResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {}
            is NavResult.Value -> {
                if (it.value) {
                    backNavigator.navigateBack()
                }
            }
        }
    }

    LaunchedEffect(Unit) {
        customListsViewModel.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CustomListLocationsSideEffect.ReturnWithResult ->
                    backNavigator.navigateBack(result = sideEffect.result)
                CustomListLocationsSideEffect.CloseScreen -> backNavigator.navigateBack()
            }
        }
    }

    val state by customListsViewModel.uiState.collectAsState()
    CustomListLocationsScreen(
        uiState = state,
        onSearchTermInput = customListsViewModel::onSearchTermInput,
        onSaveClick = customListsViewModel::save,
        onRelaySelectionClick = customListsViewModel::onRelaySelectionClick,
        onBackClick = {
            if (state.hasUnsavedChanges) {
                navigator.navigate(DiscardChangesDialogDestination) { launchSingleTop = true }
            } else {
                backNavigator.navigateBack()
            }
        }
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun CustomListLocationsScreen(
    uiState: CustomListLocationsUiState,
    onSearchTermInput: (String) -> Unit = {},
    onSaveClick: () -> Unit = {},
    onRelaySelectionClick: (RelayItem, selected: Boolean) -> Unit = { _, _ -> },
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithSmallTopBar(
        appBarTitle =
            stringResource(
                if (uiState.newList) {
                    R.string.add_locations
                } else {
                    R.string.edit_locations
                }
            ),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = { Actions(isSaveEnabled = uiState.saveEnabled, onSaveClick = onSaveClick) }
    ) { modifier ->
        Column(modifier = modifier) {
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
            Spacer(modifier = Modifier.height(Dimens.verticalSpace))
            val lazyListState = rememberLazyListState()
            LazyColumn(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier =
                    Modifier.drawVerticalScrollbar(
                            state = lazyListState,
                            color =
                                MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar)
                        )
                        .fillMaxWidth(),
                state = lazyListState,
            ) {
                when (uiState) {
                    is CustomListLocationsUiState.Loading -> {
                        loading()
                    }
                    is CustomListLocationsUiState.Content.Empty -> {
                        empty(searchTerm = uiState.searchTerm)
                    }
                    is CustomListLocationsUiState.Content.Data -> {
                        content(uiState = uiState, onRelaySelectedChanged = onRelaySelectionClick)
                    }
                }
            }
        }
    }
}

@Composable
private fun Actions(isSaveEnabled: Boolean, onSaveClick: () -> Unit) {
    TextButton(
        onClick = onSaveClick,
        enabled = isSaveEnabled,
        colors =
            ButtonDefaults.textButtonColors()
                .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
        modifier = Modifier.testTag(SAVE_BUTTON_TEST_TAG)
    ) {
        Text(
            text = stringResource(R.string.save),
        )
    }
}

private fun LazyListScope.loading() {
    item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(
            modifier = Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR)
        )
    }
}

private fun LazyListScope.empty(searchTerm: String) {
    item(key = CommonContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) {
        LocationsEmptyText(searchTerm = searchTerm)
    }
}

private fun LazyListScope.content(
    uiState: CustomListLocationsUiState.Content.Data,
    onRelaySelectedChanged: (RelayItem, selected: Boolean) -> Unit,
) {
    items(
        count = uiState.availableLocations.size,
        key = { index -> uiState.availableLocations[index].hashCode() },
        contentType = { ContentType.ITEM },
    ) { index ->
        val country = uiState.availableLocations[index]
        CheckableRelayLocationCell(
            relay = country,
            modifier = Modifier.animateContentSize(),
            onRelayCheckedChange = onRelaySelectedChanged,
            selectedRelays = uiState.selectedLocations,
        )
    }
}
