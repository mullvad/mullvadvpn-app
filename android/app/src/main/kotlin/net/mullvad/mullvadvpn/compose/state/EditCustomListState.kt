package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.relaylist.RelayItem

sealed interface EditCustomListState {
    data object Loading : EditCustomListState

    data class Content(val id: String, val name: String, val locations: List<RelayItem>) :
        EditCustomListState
}
