package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.theme.DarkThemeState

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isSupportedVersion: Boolean,
    val isPlayBuild: Boolean,
    val isMaterialYouTheme: Boolean,
    val darkThemeState: DarkThemeState
)
