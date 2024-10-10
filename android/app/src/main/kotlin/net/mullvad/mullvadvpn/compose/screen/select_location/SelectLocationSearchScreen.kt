package net.mullvad.mullvadvpn.compose.screen.select_location

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SearchSelectLocationUiState
import net.mullvad.mullvadvpn.compose.transitions.SearchTransition
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.SearchSelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewSelectLocationSearchScreen() {
    AppTheme { SelectLocationSearchScreen(state = SearchSelectLocationUiState.NoQuery("")) }
}

data class SelectLocationSearchNavArgs(val relayListSelection: RelayListSelection)

@Composable
@Destination<RootGraph>(
    style = SearchTransition::class,
    navArgs = SelectLocationSearchNavArgs::class,
)
fun SelectLocationSearch(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<SearchSelectLocationViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    SelectLocationSearchScreen(
        state = state,
        onSelectRelay = viewModel::selectRelay,
        onToggleExpand = viewModel::onToggleExpand,
        onSearchInputChanged = viewModel::onSearchInputUpdated,
        onGoBack = dropUnlessResumed { navigator.navigateUp() },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SelectLocationSearchScreen(
    state: SearchSelectLocationUiState,
    onSelectRelay: (RelayItem) -> Unit = {},
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit = { _, _, _ -> },
    onSearchInputChanged: (String) -> Unit = {},
    onGoBack: () -> Unit = {},
) {
    Scaffold {
        SearchBar(
            inputField = {
                SearchBarDefaults.InputField(
                    query = state.searchTerm,
                    onQueryChange = onSearchInputChanged,
                    onSearch = {
                        // Close keyboard
                    },
                    expanded = true,
                    onExpandedChange = {},
                )
            },
            expanded = true,
            onExpandedChange = { expanded -> if (!expanded) onGoBack() },
            modifier = Modifier.padding(it),
        ) {
            val backgroundColor = MaterialTheme.colorScheme.surface
            val lazyListState = rememberLazyListState()
            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .background(color = backgroundColor)
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                when (state) {
                    is SearchSelectLocationUiState.NoQuery -> noQuery()
                    is SearchSelectLocationUiState.Content -> {
                        relayListContent(
                            backgroundColor = backgroundColor,
                            relayListItems = state.relayListItems,
                            onSelectRelay = onSelectRelay,
                            onToggleExpand = onToggleExpand,
                        )
                    }
                }
            }
        }
    }
}

private fun LazyListScope.noQuery() {
    item(contentType = ContentType.DESCRIPTION) {
        Text(
            text = "Type at least 2 characters to start searching",
            style = MaterialTheme.typography.labelMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
