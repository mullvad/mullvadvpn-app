package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.usecase.FilterChip

data class SelectLocationUiState(
    val filterChips: List<FilterChip>,
    val multihopListSelection: RelayHopType,
    val activeMultihopMode: MultihopMode,
    val isSearchButtonEnabled: Boolean,
    val isEntryFilterButtonEnabled: Boolean,
    val isEntryFilteringEnabled: Boolean,
    val isRecentsEnabled: Boolean,
    val hopSelection: HopSelection,
    val tunnelErrorStateCause: ErrorStateCause?,
    val lastKnownLocation: String?,
    val entryCountry: String?,
    val hasAnyEntryFilter: Boolean,
    val hasAnyExitFilter: Boolean,
) {
    val multihopEnabled: Boolean = hopSelection is HopSelection.Multi
    val relayListType =
        if (multihopEnabled) RelayListType.Multihop(multihopListSelection) else RelayListType.Single
}
