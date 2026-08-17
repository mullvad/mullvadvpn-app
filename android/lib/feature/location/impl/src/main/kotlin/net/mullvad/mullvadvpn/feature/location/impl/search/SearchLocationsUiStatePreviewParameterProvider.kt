package net.mullvad.mullvadvpn.feature.location.impl.search

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemPreviewData
import net.mullvad.mullvadvpn.lib.usecase.FilterChip

class SearchLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<Unit, SearchLocationUiState, Unit>> {
    override val values =
        sequenceOf(
            Lce.Loading(Unit),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "",
                    relayListType = RelayListType.Multihop(RelayHopType.ENTRY),
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                    highlights = emptyMap(),
                )
            ),
            Lce.Error(Unit),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "Mullvad",
                    relayListType = RelayListType.Multihop(RelayHopType.ENTRY),
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateEmptyList("Mullvad", isSearching = true),
                    customLists = emptyList(),
                    highlights =  emptyMap(),
                )
            ),
            Lce.Content(
                SearchLocationUiState(
                    searchTerm = "Germany",
                    relayListType = RelayListType.Multihop(RelayHopType.ENTRY),
                    filterChips = listOf(FilterChip.Entry),
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = true,
                        ),
                    customLists = emptyList(),
                    highlights =  emptyMap(),
                )
            ),
        )
}
