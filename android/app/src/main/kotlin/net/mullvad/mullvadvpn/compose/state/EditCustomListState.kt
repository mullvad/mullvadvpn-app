package net.mullvad.mullvadvpn.compose.state

sealed interface EditCustomListState {
    data object Loading : EditCustomListState

    data class Content(val id: String, val name: String, val numberOfLocations: Int) :
        EditCustomListState
}
