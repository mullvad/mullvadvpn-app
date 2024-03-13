package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class Content(
        val searchTerm: String,
        val selectedOwnership: Ownership?,
        val selectedProvidersCount: Int?,
        val filteredCustomLists: List<RelayItem.CustomList>,
        val customLists: List<RelayItem.CustomList>,
        val countries: List<RelayItem.Country>,
        val selectedItem: RelayItem?
    ) : SelectLocationUiState {
        val hasFilter: Boolean = (selectedProvidersCount != null || selectedOwnership != null)
        val inSearch = searchTerm.length >= MIN_SEARCH_LENGTH
        val showCustomLists = inSearch.not() || filteredCustomLists.isNotEmpty()
        // Show empty state if we don't have any relays or if we are searching and no custom list or
        // relay is found
        val showEmpty = countries.isEmpty() && (inSearch.not() || filteredCustomLists.isEmpty())
    }
}
