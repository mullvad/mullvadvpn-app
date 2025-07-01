package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.util.Lce

data class CustomListLocationsUiState(
    val newList: Boolean,
    val content: Lce<Unit, CustomListLocationsData, Unit>,
)

data class CustomListLocationsData(
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
    val itemPosition: ItemPosition = ItemPosition.Single,
)
