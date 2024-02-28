package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayItem

interface CustomListsUiState {
    object Loading : CustomListsUiState

    data class Content(val customLists: List<RelayItem.CustomList> = emptyList()) :
        CustomListsUiState
}
