package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.SelectedLocation

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class Data(
        val searchTerm: String,
        val selectedOwnership: Ownership?,
        val selectedProvidersCount: Int?,
        val relayListState: RelayListState
    ) : SelectLocationUiState {
        val hasFilter: Boolean = (selectedProvidersCount != null || selectedOwnership != null)
    }
}

sealed interface RelayListState {
    data object Empty : RelayListState

    data class RelayList(
        val countries: List<RelayCountry>,
        val selectedLocation: SelectedLocation?
    ) : RelayListState
}
