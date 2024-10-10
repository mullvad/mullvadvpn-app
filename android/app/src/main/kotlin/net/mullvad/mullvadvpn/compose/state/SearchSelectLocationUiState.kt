package net.mullvad.mullvadvpn.compose.state

sealed interface SearchSelectLocationUiState {
    val searchTerm: String

    data class NoQuery(override val searchTerm: String) : SearchSelectLocationUiState

    data class Content(
        override val searchTerm: String,
        val relayListItems: List<RelayListItem>,
        val relayListSelection: RelayListSelection,
    ) : SearchSelectLocationUiState
}
