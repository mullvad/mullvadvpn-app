package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.VersionInfo
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc
import net.mullvad.mullvadvpn.viewmodel.AppInfoUiState

class AppInfoUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Unit, AppInfoUiState>> {
    override val values: Sequence<Lc<Unit, AppInfoUiState>> =
        sequenceOf(
            Lc.Loading(Unit),
            AppInfoUiState(
                    version = VersionInfo(currentVersion = "2024.9", isSupported = true),
                    isPlayBuild = true,
                )
                .toLc(),
            AppInfoUiState(
                    version = VersionInfo(currentVersion = "2024.9", isSupported = false),
                    isPlayBuild = true,
                )
                .toLc(),
        )
}
