package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

data class SelectLocationViewModelState(
    val newRelayList: RelayList? = null,
    val newSelectedItem: RelayItem? = null,
) {
    fun toUiState(): SelectLocationUiState =
        newRelayList?.let {
            SelectLocationUiState.ShowData(
                countries = newRelayList.countries,
                selectedRelay = newSelectedItem
            )
        } ?: SelectLocationUiState.Loading
}
