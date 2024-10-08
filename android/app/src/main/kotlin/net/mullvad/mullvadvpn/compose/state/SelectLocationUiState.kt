package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.FilterChip

data class SelectLocationUiState(
    val filterChips: List<FilterChip>,
    val multihopEnabled: Boolean,
    val relayListType: RelayListType,
)
