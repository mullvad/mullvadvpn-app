package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.util.Lc

class SearchLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<SearchLocationUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading,
            Lc.Content(
                SearchLocationUiState(
                    searchTerm = "",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                )
            ),
            Lc.Content(
                SearchLocationUiState(
                    searchTerm = "",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateEmptyList("", isSearching = false),
                    customLists = emptyList(),
                )
            ),
            Lc.Content(
                SearchLocationUiState(
                    searchTerm = "Mullvad",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateEmptyList("Mullvad", isSearching = true),
                    customLists = emptyList(),
                )
            ),
            Lc.Content(
                SearchLocationUiState(
                    searchTerm = "Germany",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                )
            ),
        )
}
