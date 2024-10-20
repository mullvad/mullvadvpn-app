package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.usecase.FilterChip

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class Content(
        val filterChips: List<FilterChip>,
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
        val multihopEnabled: Boolean,
        val relayListSelection: RelayListSelection,
    ) : SelectLocationUiState

    fun relayListSelection() = (this as? Content)?.relayListSelection
}
