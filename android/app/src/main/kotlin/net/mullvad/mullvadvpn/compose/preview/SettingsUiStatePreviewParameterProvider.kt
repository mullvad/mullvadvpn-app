package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState

class SettingsUiStatePreviewParameterProvider : PreviewParameterProvider<SettingsUiState> {
    override val values =
        sequenceOf(
            SettingsUiState(
                appVersion = "2222.22",
                isLoggedIn = true,
                isSupportedVersion = true,
                isDaitaEnabled = true,
                isPlayBuild = true,
                multihopEnabled = false,
            ),
            SettingsUiState(
                appVersion = "9000.1",
                isLoggedIn = false,
                isSupportedVersion = false,
                isDaitaEnabled = false,
                isPlayBuild = false,
                multihopEnabled = false,
            ),
        )
}
