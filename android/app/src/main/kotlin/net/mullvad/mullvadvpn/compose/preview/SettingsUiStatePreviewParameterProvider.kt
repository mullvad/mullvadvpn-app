package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc

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
                    multihopEnabled = false,
                )
                .toLc(),
            SettingsUiState(
                    appVersion = "9000.1",
                    isLoggedIn = false,
                    isSupportedVersion = false,
                    isDaitaEnabled = false,
                    isPlayBuild = false,
                    multihopEnabled = false,
                )
                .toLc(),
        )
}
