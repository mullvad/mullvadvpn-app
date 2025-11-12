package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.usecase.FilterChip

data class SelectLocationUiState(
    val filterChips: List<FilterChip>,
    val multihopEnabled: Boolean,
    val relayListType: RelayListType,
    val isSearchButtonEnabled: Boolean,
    val isFilterButtonEnabled: Boolean,
    val isRecentsEnabled: Boolean,
    val entrySelection: String?,
    val exitSelection: String?,
    val tunnelErrorStateCause: ErrorStateCause?,
    val entrySelectionAllowed: Boolean,
)
