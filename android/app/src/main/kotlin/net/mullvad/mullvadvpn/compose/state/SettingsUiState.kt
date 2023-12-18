package net.mullvad.mullvadvpn.compose.state

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isUpdateAvailable: Boolean,
    val isPlayBuild: Boolean
)
