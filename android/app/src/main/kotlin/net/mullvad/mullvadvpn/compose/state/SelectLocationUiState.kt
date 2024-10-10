package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem

typealias ModelOwnership = net.mullvad.mullvadvpn.lib.model.Ownership

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class Content(
        val filterChips: List<FilterChip>,
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
        val multihopEnabled: Boolean,
        val relayListSelection: RelayListSelection,
    ) : SelectLocationUiState
}

sealed interface FilterChip {
    data class Ownership(val ownership: ModelOwnership) : FilterChip

    data class Provider(val count: Int) : FilterChip

    data object Daita : FilterChip
}
