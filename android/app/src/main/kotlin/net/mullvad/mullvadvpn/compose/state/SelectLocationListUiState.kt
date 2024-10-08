package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem

sealed interface SelectLocationListUiState {

    data object Loading : SelectLocationListUiState

    data class Content(
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
    ) : SelectLocationListUiState
}
