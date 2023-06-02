package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.theme.DarkThemeState

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isUpdateAvailable: Boolean,
    val isPlayBuild: Boolean,
    val isMaterialYouTheme: Boolean,
    val darkThemeState: DarkThemeState
)
