package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemPreviewData
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.util.Lce

class SearchLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<Unit, SearchLocationUiState, Unit>> {
    override val values =
        sequenceOf(
            Lce.Loading(Unit),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                )
            ),
            Lce.Error(Unit),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "Mullvad",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateEmptyList("Mullvad", isSearching = true),
                    customLists = emptyList(),
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                )
            ),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "Germany",
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                )
            ),
        )
}
