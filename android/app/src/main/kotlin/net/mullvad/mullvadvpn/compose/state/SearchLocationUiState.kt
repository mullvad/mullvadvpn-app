package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.usecase.FilterChip

data class SearchLocationUiState(
    val searchTerm: String,
    val relayListType: RelayListType,
    val filterChips: List<FilterChip>,
    val relayListItems: List<RelayListItem>,
    val customLists: List<RelayItem.CustomList>,
)
