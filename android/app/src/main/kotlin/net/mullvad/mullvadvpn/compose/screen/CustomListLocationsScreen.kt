package net.mullvad.mullvadvpn.compose.screen

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.DiscardChangesDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.EmptyRelayListText
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.dialog.info.Confirmed
import net.mullvad.mullvadvpn.compose.extensions.animateScrollAndCentralizeItem
import net.mullvad.mullvadvpn.compose.preview.CustomListLocationUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.screen.location.positionalPadding
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsData
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayLocationCell
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsSideEffect
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Empty|Loading")
@Composable
private fun PreviewCustomListLocationScreen(
    @PreviewParameter(CustomListLocationUiStatePreviewParameterProvider::class)
    state: CustomListLocationsUiState
) {
    AppTheme {
        CustomListLocationsScreen(
            state = state,
            onSearchTermInput = {},
            onSaveClick = {},
            onRelaySelectionClick = { _, _ -> },
            onExpand = { _, _ -> },
            onBackClick = {},
        )
    }
}

data class CustomListLocationsNavArgs(val customListId: CustomListId, val newList: Boolean)

@Composable
@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = CustomListLocationsNavArgs::class,
)
fun CustomListLocations(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListActionResultData>,
    discardChangesResultRecipient: ResultRecipient<DiscardChangesDestination, Confirmed>,
) {
    val customListsViewModel = koinViewModel<CustomListLocationsViewModel>()

    discardChangesResultRecipient.OnNavResultValue { backNavigator.navigateBack() }

    CollectSideEffectWithLifecycle(customListsViewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is CustomListLocationsSideEffect.ReturnWithResultData ->
                backNavigator.navigateBack(result = sideEffect.result)
        }
    }

    val state by customListsViewModel.uiState.collectAsStateWithLifecycle()
    CustomListLocationsScreen(
        state = state,
        onSearchTermInput = customListsViewModel::onSearchTermInput,
        onSaveClick = customListsViewModel::save,
        onRelaySelectionClick = customListsViewModel::onRelaySelectionClick,
        onExpand = customListsViewModel::onExpand,
        onBackClick =
            dropUnlessResumed {
                if (state.content.contentOrNull()?.hasUnsavedChanges == true) {
                    navigator.navigate(DiscardChangesDestination)
                } else {
                    backNavigator.navigateBack()
                }
            },
    )
}

@Composable
fun CustomListLocationsScreen(
    state: CustomListLocationsUiState,
    onSearchTermInput: (String) -> Unit,
    onSaveClick: () -> Unit,
    onRelaySelectionClick: (RelayItem.Location, selected: Boolean) -> Unit,
    onExpand: (RelayItem.Location, selected: Boolean) -> Unit,
    onBackClick: () -> Unit,
) {
    BackHandler(onBack = onBackClick)

    ScaffoldWithSmallTopBar(
        appBarTitle =
            stringResource(
                if (state.newList) {
                    R.string.add_locations
                } else {
                    R.string.edit_locations
                }
            ),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            Actions(
                isSaveEnabled = state.content.contentOrNull()?.saveEnabled == true,
                onSaveClick = onSaveClick,
            )
        },
    ) { modifier ->
        Column(modifier = modifier) {
            SearchTextField(
                modifier =
                    Modifier.fillMaxWidth()
                        .height(Dimens.searchFieldHeight)
                        .padding(horizontal = Dimens.mediumPadding),
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
                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        )
                        .padding(horizontal = Dimens.mediumPadding)
                        .fillMaxSize(),
                state = lazyListState,
            ) {
                when (state.content) {
                    is Lce.Loading -> {
                        loading()
                    }

                    is Lce.Error -> {
                        empty()
                    }

                    is Lce.Content -> {
                        content(
                            uiState = state.content.value,
                            onRelaySelectedChanged = onRelaySelectionClick,
                            onExpand = onExpand,
                        )
                    }
                }
            }

            if (state.content is Lce.Content && !state.newList) {
                val firstChecked = state.content.value.locations.indexOfFirst { it.checked }
                LaunchedEffect(Unit) {
                    if (firstChecked != -1) {
                        lazyListState.scrollToItem(firstChecked)
                        lazyListState.animateScrollAndCentralizeItem(firstChecked)
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
        modifier = Modifier.testTag(SAVE_BUTTON_TEST_TAG),
    ) {
        Text(text = stringResource(R.string.save), style = MaterialTheme.typography.labelLarge)
    }
}

private fun LazyListScope.loading() {
    item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge()
    }
}

private fun LazyListScope.empty() {
    item(key = CommonContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) {
        EmptyRelayListText()
    }
}

private fun LazyListScope.content(
    uiState: CustomListLocationsData,
    onExpand: (RelayItem.Location, expand: Boolean) -> Unit,
    onRelaySelectedChanged: (RelayItem.Location, selected: Boolean) -> Unit,
) {
    if (uiState.locations.isEmpty()) {
        item(key = CommonContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) {
            LocationsEmptyText(searchTerm = uiState.searchTerm)
        }
    } else {
        items(uiState.locations, key = { listItem -> listItem.item.id }) { listItem ->
            CheckableRelayLocationCell(
                modifier = Modifier.animateItem().positionalPadding(listItem.itemPosition),
                item = listItem,
                onRelayCheckedChange = { isChecked ->
                    onRelaySelectedChanged(listItem.item, isChecked)
                },
                onExpand = { expand -> onExpand(listItem.item, expand) },
            )
        }
    }
}
