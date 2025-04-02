package net.mullvad.mullvadvpn.compose.state

data class DaitaUiState(
    val daitaEnabled: Boolean,
    val directOnly: Boolean,
    val isModal: Boolean = false,
)
