package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.usecase.FilterChip

sealed interface SearchLocationUiState {
    val searchTerm: String
    val filterChips: List<FilterChip>

    data class NoQuery(
        override val searchTerm: String,
        override val filterChips: List<FilterChip>,
    ) : SearchLocationUiState

    data class Content(
        override val searchTerm: String,
        override val filterChips: List<FilterChip>,
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
        val relayListType: RelayListType,
    ) : SearchLocationUiState
}
