package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.viewmodel.AppInfoUiState

class AppInfoUiStatePreviewParameterProvider : PreviewParameterProvider<AppInfoUiState> {
    override val values: Sequence<AppInfoUiState> =
        sequenceOf(
            AppInfoUiState(
                version = VersionInfo(currentVersion = "2024.9", isSupported = true),
                changes = listOf("More betterer app"),
                isPlayBuild = true,
            ),
            AppInfoUiState(
                version = VersionInfo(currentVersion = "2024.9", isSupported = false),
                changes = listOf("More betterer app"),
                isPlayBuild = true,
            ),
        )
}
