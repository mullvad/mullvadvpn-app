package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.intl.Locale
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.toLowerCase
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.animateScrollAndCentralizeItem
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.util.RunOnKeyChange
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationListViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
fun SelectLocationList(
    backgroundColor: Color,
    relayListType: RelayListType,
    onSelectRelay: (RelayItem) -> Unit,
    openDaitaSettings: () -> Unit,
    onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit,
) {
    val viewModel =
        koinViewModel<SelectLocationListViewModel>(
            key = relayListType.name,
            parameters = { parametersOf(relayListType) },
        )
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val lazyListState = rememberLazyListState()
    val stateActual = state
    RunOnKeyChange(stateActual is SelectLocationListUiState.Content) {
        stateActual.indexOfSelectedRelayItem()?.let { index ->
            lazyListState.scrollToItem(index)
            lazyListState.animateScrollAndCentralizeItem(index)
        }
    }
    LazyColumn(
        modifier =
            Modifier.fillMaxSize()
                .drawVerticalScrollbar(
                    lazyListState,
                    MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                ),
        state = lazyListState,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement =
            if (state is SelectLocationListUiState.EntryBlocked) {
                Arrangement.Center
            } else {
                Arrangement.Top
            },
    ) {
        when (stateActual) {
            SelectLocationListUiState.Loading -> {
                loading()
            }
            SelectLocationListUiState.EntryBlocked -> {
                entryBlocked(openDaitaSettings = openDaitaSettings)
            }
            is SelectLocationListUiState.Content -> {
                relayListContent(
                    backgroundColor = backgroundColor,
                    relayListItems = stateActual.relayListItems,
                    customLists = stateActual.customLists,
                    onSelectRelay = onSelectRelay,
                    onToggleExpand = viewModel::onToggleExpand,
                    onUpdateBottomSheetState = onUpdateBottomSheetState,
                )
            }
        }
    }
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR))
    }
}

private fun LazyListScope.entryBlocked(openDaitaSettings: () -> Unit) {
    item(contentType = ContentType.DESCRIPTION) {
        Text(
            text =
                stringResource(
                    R.string.multihop_entry_disabled_description,
                    stringResource(R.string.multihop).toLowerCase(Locale.current),
                    stringResource(id = R.string.daita),
                    stringResource(R.string.direct_only),
                    stringResource(id = R.string.daita),
                ),
            style = MaterialTheme.typography.labelMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        )
    }
    item(contentType = ContentType.SPACER) {
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
    }
    item(contentType = ContentType.BUTTON) {
        PrimaryButton(
            text =
                stringResource(R.string.open_feature_settings, stringResource(id = R.string.daita)),
            onClick = openDaitaSettings,
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
        )
    }
}

private fun SelectLocationListUiState.indexOfSelectedRelayItem(): Int? =
    if (this is SelectLocationListUiState.Content) {
        val index =
            relayListItems.indexOfFirst {
                when (it) {
                    is RelayListItem.CustomListItem -> it.isSelected
                    is RelayListItem.GeoLocationItem -> it.isSelected
                    is RelayListItem.CustomListEntryItem,
                    is RelayListItem.CustomListFooter,
                    RelayListItem.CustomListHeader,
                    RelayListItem.LocationHeader,
                    is RelayListItem.LocationsEmptyText -> false
                }
            }
        if (index >= 0) index else null
    } else {
        null
    }
