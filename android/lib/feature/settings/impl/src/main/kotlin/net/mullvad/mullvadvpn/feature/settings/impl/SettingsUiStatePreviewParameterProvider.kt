package net.mullvad.mullvadvpn.feature.settings.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc

class SettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, SettingsUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            SettingsUiState(
                    appVersion = "2222.22",
                    deviceState = DeviceState.LoggedIn,
                    isSupportedVersion = true,
                    isDaitaEnabled = true,
                    isPlayBuild = true,
                    multihopEnabled = false,
                )
                .toLc(),
            SettingsUiState(
                    appVersion = "9000.1",
                    deviceState = DeviceState.LoggedOut,
                    isSupportedVersion = false,
                    isDaitaEnabled = false,
                    isPlayBuild = false,
                    multihopEnabled = false,
                )
                .toLc(),
            SettingsUiState(
                    appVersion = "9000.1",
                    deviceState = DeviceState.Revoked,
                    isSupportedVersion = false,
                    isDaitaEnabled = false,
                    isPlayBuild = false,
                    multihopEnabled = false,
                )
                .toLc(),
        )
}
