package net.mullvad.mullvadvpn.feature.settings.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.MultihopMode

class SettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, SettingsUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            SettingsUiState(
                    appVersion = "2222.22",
                    isLoggedIn = true,
                    isSupportedVersion = true,
                    isDaitaEnabled = true,
                    isPlayBuild = true,
                    splitTunnelingIsActive = true,
                    multihopMode = MultihopMode.NEVER,
                )
                .toLc(),
            SettingsUiState(
                    appVersion = "9000.1",
                    isLoggedIn = false,
                    isSupportedVersion = false,
                    isDaitaEnabled = false,
                    isPlayBuild = false,
                    multihopMode = MultihopMode.NEVER,
                    splitTunnelingIsActive = false,
                )
                .toLc(),
            SettingsUiState(
                    appVersion = "9000.1",
                    isLoggedIn = false,
                    isSupportedVersion = false,
                    isDaitaEnabled = false,
                    isPlayBuild = false,
                    splitTunnelingIsActive = true,
                    multihopMode = MultihopMode.NEVER,
                )
                .toLc(),
        )
}
