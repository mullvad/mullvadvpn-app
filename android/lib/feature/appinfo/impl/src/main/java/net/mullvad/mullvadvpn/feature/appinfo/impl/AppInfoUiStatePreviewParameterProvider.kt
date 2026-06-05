package net.mullvad.mullvadvpn.feature.appinfo.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration
import net.mullvad.mullvadvpn.lib.model.VersionInfo

class AppInfoUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Unit, AppInfoUiState>> {
    override val values: Sequence<Lc<Unit, AppInfoUiState>> =
        sequenceOf(
            Lc.Loading(Unit),
            AppInfoUiState(
                    version = VersionInfo(currentVersion = "2024.9", isSupported = true),
                    isPlayBuild = true,
                    splitFilterMigration = null,
                )
                .toLc(),
            AppInfoUiState(
                    version = VersionInfo(currentVersion = "2024.9", isSupported = false),
                    isPlayBuild = true,
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            filtersSet = true,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                        ),
                )
                .toLc(),
        )
}
