package net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists

import net.mullvad.mullvadvpn.lib.model.CustomList

interface CustomListsUiState {
    object Loading : CustomListsUiState

    data class Content(val customLists: List<CustomList> = emptyList()) : CustomListsUiState
}
