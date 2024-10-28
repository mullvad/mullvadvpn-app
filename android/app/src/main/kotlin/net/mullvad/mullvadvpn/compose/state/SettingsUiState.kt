package net.mullvad.mullvadvpn.compose.state

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isSupportedVersion: Boolean,
    val isPlayBuild: Boolean,
    val multihopEnabled: Boolean,
)
