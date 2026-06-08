package net.mullvad.mullvadvpn.feature.settings.impl

data class SettingsUiState(
    val appVersion: String,
    val deviceState: DeviceState?,
    val isSupportedVersion: Boolean,
    val isDaitaEnabled: Boolean,
    val isPlayBuild: Boolean,
    val multihopEnabled: Boolean,
)

sealed interface DeviceState {
    data object LoggedOut : DeviceState

    data object Revoked : DeviceState

    data object LoggedIn : DeviceState
}
