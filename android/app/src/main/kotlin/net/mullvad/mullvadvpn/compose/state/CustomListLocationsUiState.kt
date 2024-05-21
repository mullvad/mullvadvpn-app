package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.RelayItem

sealed interface CustomListLocationsUiState {
    val newList: Boolean
    val saveEnabled: Boolean
    val hasUnsavedChanges: Boolean

    data class Loading(override val newList: Boolean = false) : CustomListLocationsUiState {
        override val saveEnabled: Boolean = false
        override val hasUnsavedChanges: Boolean = false
    }

    sealed interface Content : CustomListLocationsUiState {
        val searchTerm: String

        data class Empty(override val newList: Boolean, override val searchTerm: String) : Content {
            override val saveEnabled: Boolean = false
            override val hasUnsavedChanges: Boolean = false
        }

        data class Data(
            override val newList: Boolean = false,
            val availableLocations: List<RelayItem.Location.Country> = emptyList(),
            val selectedLocations: Set<RelayItem> = emptySet(),
            override val searchTerm: String = "",
            override val saveEnabled: Boolean = false,
            override val hasUnsavedChanges: Boolean = false
        ) : Content
    }
}
