package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.CustomList

interface CustomListsUiState {
    object Loading : CustomListsUiState

    data class Content(val customLists: List<CustomList> = emptyList()) : CustomListsUiState
}
