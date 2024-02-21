package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface CustomListLocationsUiState {
    val newList: Boolean

    data class Loading(override val newList: Boolean = false) : CustomListLocationsUiState

    sealed interface Content : CustomListLocationsUiState {
        val searchTerm: String

        data class Empty(override val newList: Boolean, override val searchTerm: String) : Content

        data class Data(
            override val newList: Boolean = false,
            val availableLocations: List<RelayItem.Country> = emptyList(),
            val selectedLocations: Set<RelayItem> = emptySet(),
            override val searchTerm: String = ""
        ) : Content
    }
}
