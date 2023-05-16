package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

data class SelectLocationViewModelState(
    val newRelayList: RelayList? = null,
    val newSelectedItem: RelayItem? = null,
    val close: Boolean = false
) {
    fun toUiState(): SelectLocationUiState =
        when {
            close ->
                SelectLocationUiState.Data.Close(
                    countries = newRelayList?.countries ?: emptyList(),
                    selectedRelay = newSelectedItem
                )
            newRelayList != null -> {
                SelectLocationUiState.Data.Show(
                    countries = newRelayList.countries,
                    selectedRelay = newSelectedItem
                )
            }
            else -> SelectLocationUiState.Loading
        }
}
