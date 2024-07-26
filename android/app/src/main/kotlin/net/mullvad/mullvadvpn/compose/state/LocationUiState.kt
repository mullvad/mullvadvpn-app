package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem

sealed interface LocationUiState {
    val name: String

    data class Loading(override val name: String) : LocationUiState

    data class Content(val location: RelayItem.Location, val customLists: List<CustomListEntry>) :
        LocationUiState {
        override val name: String = location.name
    }
}

data class CustomListEntry(val customList: RelayItem.CustomList, val canAdd: Boolean)
