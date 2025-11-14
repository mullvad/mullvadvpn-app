package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.usecase.FilterChip

data class SelectLocationUiState(
    val filterChips: List<FilterChip>,
    val relayListType: RelayListType,
    val isSearchButtonEnabled: Boolean,
    val isFilterButtonEnabled: Boolean,
    val isRecentsEnabled: Boolean,
    val hopSelection: HopSelection,
    val tunnelErrorStateCause: ErrorStateCause?,
) {
    val multihopEnabled: Boolean = hopSelection is HopSelection.Multi
}
