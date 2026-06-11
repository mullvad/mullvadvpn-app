package net.mullvad.mullvadvpn.feature.settings.impl

import net.mullvad.mullvadvpn.lib.model.MultihopMode

data class SettingsUiState(
    val appVersion: String,
    val isLoggedIn: Boolean,
    val isSupportedVersion: Boolean,
    val isDaitaEnabled: Boolean,
    val isPlayBuild: Boolean,
    val multihopMode: MultihopMode,
)
