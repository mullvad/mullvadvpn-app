package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class ShowData(
        val searchTerm: String,
        val countries: List<RelayCountry>,
        val selectedRelay: RelayItem?,
        val selectedOwnership: Ownership?,
        val selectedProvidersCount: Int?,
    ) : SelectLocationUiState {
        val hasFilter: Boolean = (selectedProvidersCount != null || selectedOwnership != null)
    }
}
