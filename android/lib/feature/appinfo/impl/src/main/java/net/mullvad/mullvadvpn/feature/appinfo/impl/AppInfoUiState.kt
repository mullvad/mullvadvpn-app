package net.mullvad.mullvadvpn.feature.appinfo.impl

import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration
import net.mullvad.mullvadvpn.lib.model.VersionInfo

data class AppInfoUiState(
    val version: VersionInfo,
    val splitFilterMigration: SplitFilterMigration?,
    val isPlayBuild: Boolean,
)
