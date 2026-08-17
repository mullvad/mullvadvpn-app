package net.mullvad.mullvadvpn.feature.customlist.impl.screen.editlocations

import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SearchMatch
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem

data class CustomListLocationsUiState(
    val newList: Boolean,
    val content: Lce<Unit, CustomListLocationsData, Unit>,
)

data class CustomListLocationsData(
    val saveEnabled: Boolean,
    val hasUnsavedChanges: Boolean,
    val searchTerm: String,
    val locations: List<CheckableRelayListItem>,
    val highlights: Map<RelayItemId, SearchMatch>
)
