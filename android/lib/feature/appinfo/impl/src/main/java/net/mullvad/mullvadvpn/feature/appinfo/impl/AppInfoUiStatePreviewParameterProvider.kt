package net.mullvad.mullvadvpn.feature.appinfo.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.VersionInfo

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
