package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface SelectLocationUiState {
    data object Loading : SelectLocationUiState

    data class ShowData(val countries: List<RelayCountry>, val selectedRelay: RelayItem?) :
        SelectLocationUiState

    data class NoSearchResultFound(val searchTerm: String) : SelectLocationUiState
}
