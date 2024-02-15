package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.CheckableRelayLocationCell
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsSideEffect
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview
fun PreviewCustomListsScreen() {
    AppTheme {
        CustomListLocationsScreen(
            newList = true,
            uiState = CustomListLocationsUiState.Content.Data()
        )
    }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun CustomListLocations(navigator: DestinationsNavigator, customListKey: String, newList: Boolean) {
    val customListsViewModel =
        koinViewModel<CustomListLocationsViewModel>(parameters = { parametersOf(customListKey) })

    LaunchedEffect(Unit) {
        customListsViewModel.uiSideEffect.collect { sideEffect ->
            when (sideEffect) {
                is CustomListLocationsSideEffect.CloseScreen -> navigator.navigateUp()
            }
        }
    }

    val state by customListsViewModel.uiState.collectAsState()
    CustomListLocationsScreen(
        newList = newList,
        uiState = state,
        onSaveClick = customListsViewModel::save,
        onSelectLocationClick = customListsViewModel::selectLocation,
        onDeselectLocationClick = customListsViewModel::deselectLocation,
        onBackClick = navigator::navigateUp
    )
}

@Composable
fun CustomListLocationsScreen(
    newList: Boolean,
    uiState: CustomListLocationsUiState,
    onSaveClick: () -> Unit = {},
    onSelectLocationClick: (RelayItem) -> Unit = {},
    onDeselectLocationClick: (RelayItem) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold(
        modifier = Modifier.background(backgroundColor).systemBarsPadding().fillMaxSize(),
        topBar = {
            CustomListLocationsTopBar(
                newList = newList,
                onBackClick = onBackClick,
                onSaveClick = onSaveClick
            )
        },
        content = { contentPadding ->
            LazyColumn(modifier = Modifier.padding(contentPadding).fillMaxSize()) {
                when (uiState) {
                    is CustomListLocationsUiState.Loading -> {
                        loading()
                    }
                    is CustomListLocationsUiState.Content.Empty -> {
                        empty(searchTerm = uiState.searchTerm)
                    }
                    is CustomListLocationsUiState.Content.Data -> {
                        content(
                            uiState = uiState,
                            onRelaySelected = onSelectLocationClick,
                            onRelayDeselected = onDeselectLocationClick
                        )
                    }
                }
            }
        }
    )
}

@Composable
private fun CustomListLocationsTopBar(
    newList: Boolean,
    onBackClick: () -> Unit,
    onSaveClick: () -> Unit
) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onClick = onBackClick) {
            Icon(
                painter = painterResource(id = R.drawable.icon_back),
                contentDescription = null,
                tint = Color.Unspecified,
            )
        }
        Text(
            text =
                stringResource(
                    if (newList) {
                        R.string.add_locations
                    } else {
                        R.string.edit_locations
                    }
                ),
            modifier = Modifier.weight(1f).padding(end = Dimens.titleIconSize),
            textAlign = TextAlign.Start,
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onPrimary
        )
        IconButton(onClick = {}) {
            Icon(
                painter = painterResource(id = R.drawable.icons_search),
                contentDescription = null,
                tint = Color.Unspecified,
            )
        }
        TextButton(onClick = onSaveClick) {
            Text(text = stringResource(R.string.save), color = MaterialTheme.colorScheme.onPrimary)
        }
    }
}

private fun LazyListScope.loading() {
    item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge()
    }
}

private fun LazyListScope.empty(searchTerm: String) {
    item(key = CommonContentKey.EMPTY, contentType = ContentType.EMPTY_TEXT) {
        LocationsEmptyText(searchTerm = searchTerm)
    }
}

private fun LazyListScope.content(
    uiState: CustomListLocationsUiState.Content.Data,
    onRelaySelected: (RelayItem) -> Unit,
    onRelayDeselected: (RelayItem) -> Unit
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
            onRelaySelected = onRelaySelected,
            onRelayDeselected = onRelayDeselected,
            selectedRelays = uiState.selectedLocations,
        )
    }
}
