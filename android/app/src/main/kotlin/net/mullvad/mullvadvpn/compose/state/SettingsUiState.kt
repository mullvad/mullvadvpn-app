package net.mullvad.mullvadvpn.compose.state

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isUnsupportedVersion: Boolean,
    val isPlayBuild: Boolean
)
