package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.usecase.FilterChip

sealed interface SearchSelectLocationUiState {
    val searchTerm: String
    val filterChips: List<FilterChip>

    data class NoQuery(
        override val searchTerm: String,
        override val filterChips: List<FilterChip>,
    ) : SearchSelectLocationUiState

    data class Content(
        override val searchTerm: String,
        override val filterChips: List<FilterChip>,
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
        val relayListType: RelayListType,
    ) : SearchSelectLocationUiState
}
