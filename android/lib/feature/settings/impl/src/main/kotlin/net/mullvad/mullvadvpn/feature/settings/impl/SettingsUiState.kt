package net.mullvad.mullvadvpn.feature.settings.impl

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isSupportedVersion: Boolean,
    val isDaitaEnabled: Boolean,
    val isPlayBuild: Boolean,
    val multihopEnabled: Boolean,
)
