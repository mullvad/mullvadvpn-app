package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.FilterChip

sealed interface SelectLocationUiState {
    data object Loading : SelectLocationUiState

    data class Data(
        val filterChips: List<FilterChip>,
        val multihopEnabled: Boolean,
        val relayListType: RelayListType,
    ) : SelectLocationUiState
}
