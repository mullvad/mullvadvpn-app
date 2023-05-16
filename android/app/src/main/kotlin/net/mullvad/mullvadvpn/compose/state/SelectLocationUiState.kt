package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface SelectLocationUiState {
    object Loading : SelectLocationUiState
    sealed class Data(val countries: List<RelayCountry>, val selectedRelay: RelayItem?) :
        SelectLocationUiState {
        class Show(countries: List<RelayCountry>, selectedRelay: RelayItem?) :
            Data(countries, selectedRelay)
        class Close(countries: List<RelayCountry>, selectedRelay: RelayItem?) :
            Data(countries, selectedRelay)
    }
}
