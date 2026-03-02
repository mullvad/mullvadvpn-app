package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.usecase.FilterChip

data class SelectLocationUiState(
    val filterChips: List<FilterChip>,
    val multihopListSelection: MultihopRelayListType,
    val isSearchButtonEnabled: Boolean,
    val isFilterButtonEnabled: Boolean,
    val isRecentsEnabled: Boolean,
    val hopSelection: HopSelection,
    val tunnelErrorStateCause: ErrorStateCause?,
) {
    val multihopEnabled: Boolean = hopSelection is HopSelection.Multi
    val relayListType =
        if (multihopEnabled) RelayListType.Multihop(multihopListSelection) else RelayListType.Single
}

sealed interface SUiState {
    val hopSelection: HopSelection
    val tunnelErrorStateCause: ErrorStateCause?
    val hopList: HopList
    val filterChips: List<FilterChip>

    val isSearchButtonEnabled: Boolean
    val isFilterButtonEnabled: Boolean
    val isRecentsEnabled: Boolean

    fun multihopEnabled() = hopSelection is HopSelection.Multi
}

data class SSingleUiState(
    override val hopSelection: HopSelection.Single,
    override val filterChips: List<FilterChip>,
    override val hopList: Singlehop,
    override val tunnelErrorStateCause: ErrorStateCause?,
    override val isSearchButtonEnabled: Boolean,
    override val isFilterButtonEnabled: Boolean,
    override val isRecentsEnabled: Boolean,
) : SUiState

data class SAutohopUiState(
    override val hopSelection: HopSelection.Multi,
    override val filterChips: List<FilterChip>,
    override val hopList: Autohop,
    override val tunnelErrorStateCause: ErrorStateCause?,
    val multihopListSelection: MultihopRelayListType,
    override val isSearchButtonEnabled: Boolean,
    override val isFilterButtonEnabled: Boolean,
    override val isRecentsEnabled: Boolean,
) : SUiState

data class SMultiUiState(
    override val hopSelection: HopSelection.Multi,
    override val filterChips: List<FilterChip>,
    override val hopList: Multihop,
    override val tunnelErrorStateCause: ErrorStateCause?,
    val multihopListSelection: MultihopRelayListType,
    override val isSearchButtonEnabled: Boolean,
    override val isFilterButtonEnabled: Boolean,
    override val isRecentsEnabled: Boolean,
) : SUiState

sealed interface HopList

data class Singlehop(val exitList: List<RelayListItem>) : HopList

data class Autohop(val exitList: List<RelayListItem>) : HopList

data class Multihop(val entryList: List<RelayListItem>, val exitList: List<RelayListItem>) : HopList
