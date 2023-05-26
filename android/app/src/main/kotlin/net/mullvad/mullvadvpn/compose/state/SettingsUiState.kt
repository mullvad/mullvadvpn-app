package net.mullvad.mullvadvpn.compose.state

data class SettingsUiState(
    val isLoggedIn: Boolean,
    val appVersion: String,
    val isUpdateAvailable: Boolean
)
