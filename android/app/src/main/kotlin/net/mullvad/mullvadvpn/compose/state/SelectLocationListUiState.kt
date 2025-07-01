package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem

data class SelectLocationListUiState(
    val relayListItems: List<RelayListItem>,
    val customLists: List<RelayItem.CustomList>,
)
