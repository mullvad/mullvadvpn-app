package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.usecase.FilterChip

data class SelectLocationUiState(
    val multihopListSelection: MultihopRelayListType,
    val filterChips: Map<RelayListType, List<FilterChip>>,
    val isSearchButtonEnabled: Boolean,
    val isFilterButtonEnabled: Boolean,
    val isRecentsEnabled: Boolean,
    val hopSelection: HopSelection,
    val tunnelErrorStateCause: ErrorStateCause?,
    val entrySelectionAllowed: Boolean,
) {
    val multihopEnabled: Boolean = hopSelection is HopSelection.Multi
    val relayListType =
        if (multihopEnabled) RelayListType.Multihop(multihopListSelection) else RelayListType.Single
}
