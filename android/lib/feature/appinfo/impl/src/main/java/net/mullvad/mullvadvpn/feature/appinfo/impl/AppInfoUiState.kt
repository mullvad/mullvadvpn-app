package net.mullvad.mullvadvpn.feature.appinfo.impl

import net.mullvad.mullvadvpn.lib.model.VersionInfo

data class AppInfoUiState(val version: VersionInfo, val isPlayBuild: Boolean)
