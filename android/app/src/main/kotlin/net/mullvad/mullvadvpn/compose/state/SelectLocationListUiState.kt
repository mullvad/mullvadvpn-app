package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem

data class SelectLocationListUiState(
    val relayListItems: List<RelayListItem>,
    val customLists: List<RelayItem.CustomList>,
)
