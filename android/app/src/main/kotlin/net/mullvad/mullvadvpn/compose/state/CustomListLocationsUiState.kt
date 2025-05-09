package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem

data class CustomListLocationsUiState(
    val newList: Boolean,
    val saveEnabled: Boolean,
    val hasUnsavedChanges: Boolean,
    val searchTerm: String,
    val locations: List<RelayLocationListItem>,
)

data class RelayLocationListItem(
    val item: RelayItem.Location,
    val depth: Int = 0,
    val checked: Boolean = false,
    val expanded: Boolean = false,
)
