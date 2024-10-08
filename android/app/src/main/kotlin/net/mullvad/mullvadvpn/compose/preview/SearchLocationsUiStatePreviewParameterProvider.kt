package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip

class SearchLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<SearchLocationUiState> {
    override val values =
        sequenceOf(
            SearchLocationUiState.NoQuery(searchTerm = "", filterChips = listOf(FilterChip.Entry)),
            SearchLocationUiState.Content(
                searchTerm = "Mullvad",
                filterChips = listOf(FilterChip.Entry),
                relayListItems = RelayListItemPreviewData.generateEmptyList("Mullvad"),
                customLists = emptyList(),
            ),
            SearchLocationUiState.Content(
                searchTerm = "Germany",
                filterChips = listOf(FilterChip.Entry),
                relayListItems =
                    RelayListItemPreviewData.generateRelayListItems(
                        includeCustomLists = true,
                        isSearching = true,
                    ),
                customLists = emptyList(),
            ),
        )
}
