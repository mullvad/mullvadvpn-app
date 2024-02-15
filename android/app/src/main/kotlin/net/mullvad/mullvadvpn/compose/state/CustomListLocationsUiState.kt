package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface CustomListLocationsUiState {
    object Loading : CustomListLocationsUiState

    sealed interface Content : CustomListLocationsUiState {
        val searchTerm: String

        data class Empty(override val searchTerm: String) : Content

        data class Data(
            val availableLocations: List<RelayItem.Country> = emptyList(),
            val selectedLocations: Set<RelayItem> = emptySet(),
            override val searchTerm: String = ""
        ) : Content
    }
}
