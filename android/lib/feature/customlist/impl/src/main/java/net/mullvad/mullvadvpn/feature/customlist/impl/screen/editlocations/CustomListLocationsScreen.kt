package net.mullvad.mullvadvpn.feature.customlist.impl.screen.editlocations

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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.animateScrollAndCentralizeItem
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.DiscardCustomListChangesConfirmedNavResult
import net.mullvad.mullvadvpn.feature.customlist.api.DiscardCustomListChangesNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListLocationsNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNavResult
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists.ContentType
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

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

@Composable
fun CustomListLocations(navArgs: EditCustomListLocationsNavKey, navigator: Navigator) {
    val customListsViewModel = koinViewModel<CustomListLocationsViewModel> { parametersOf(navArgs) }

    LocalResultStore.current.consumeResult<DiscardCustomListChangesConfirmedNavResult> {
        navigator.goBack()
    }

    CollectSideEffectWithLifecycle(customListsViewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is CustomListLocationsSideEffect.ReturnWithResultData ->
                navigator.goBack(result = EditCustomListNavResult(sideEffect.result))
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
                    navigator.navigate(DiscardCustomListChangesNavKey)
                } else {
                    navigator.goBack()
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
            var searchTerm by rememberSaveable() { mutableStateOf("") }
            SearchTextField(
                modifier =
                    Modifier.fillMaxWidth()
                        .height(Dimens.searchFieldHeight)
                        .padding(horizontal = Dimens.mediumPadding),
                backgroundColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                textColor = MaterialTheme.colorScheme.onSurfaceVariant,
            ) { searchString ->
                onSearchTermInput.invoke(searchString)
                searchTerm = searchString
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
                            searchTerm = searchTerm,
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
    item(key = ContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge()
    }
}

private fun LazyListScope.empty() {
    item(key = ContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) { EmptyRelayListText() }
}

private fun LazyListScope.content(
    uiState: CustomListLocationsData,
    searchTerm: String,
    onExpand: (RelayItem.Location, expand: Boolean) -> Unit,
    onRelaySelectedChanged: (RelayItem.Location, selected: Boolean) -> Unit,
) {
    if (uiState.locations.isEmpty()) {
        item(key = ContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) {
            LocationsEmptyText(searchTerm = uiState.searchTerm)
        }
    } else {
        items(uiState.locations, key = { listItem -> listItem.item.id }) { listItem ->
            CheckableRelayListItem(
                modifier = Modifier.animateItem().positionalPadding(listItem.itemPosition),
                item = listItem,
                searchTerm = searchTerm,
                onRelayCheckedChange = { isChecked ->
                    onRelaySelectedChanged(listItem.item, isChecked)
                },
                onExpand = { expand -> onExpand(listItem.item, expand) },
            )
        }
    }
}

@Composable
private fun LocationsEmptyText(searchTerm: String) {
    Text(
        text = stringResource(R.string.search_no_matches_for_text, searchTerm),
        style = MaterialTheme.typography.bodyMedium,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.padding(Dimens.cellVerticalSpacing),
    )
}

// TODO Should maybe be in design system? Maybe if we use the correct composable we have it already?
@Composable
fun Modifier.positionalPadding(itemPosition: Position): Modifier =
    when (itemPosition) {
        Position.Top,
        Position.Single -> padding(top = Dimens.miniPadding)
        Position.Middle -> padding(top = Dimens.listItemDivider)
        Position.Bottom -> padding(top = Dimens.listItemDivider, bottom = Dimens.miniPadding)
    }
