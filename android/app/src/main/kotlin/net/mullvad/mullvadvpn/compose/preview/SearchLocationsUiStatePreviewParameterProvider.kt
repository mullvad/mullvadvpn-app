package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip

class SearchLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<SearchLocationUiState> {
    override val values =
        sequenceOf(
            SearchLocationUiState.Content(
                searchTerm = "",
                filterChips = listOf(FilterChip.Entry),
                relayListItems =
                    RelayListItemPreviewData.generateRelayListItems(
                        includeCustomLists = true,
                        isSearching = true,
                    ),
                customLists = emptyList(),
            ),
            SearchLocationUiState.Content(
                searchTerm = "",
                filterChips = listOf(FilterChip.Entry),
                relayListItems =
                    RelayListItemPreviewData.generateEmptyList("", isSearching = false),
                customLists = emptyList(),
            ),
            SearchLocationUiState.Content(
                searchTerm = "Mullvad",
                filterChips = listOf(FilterChip.Entry),
                relayListItems =
                    RelayListItemPreviewData.generateEmptyList("Mullvad", isSearching = true),
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
