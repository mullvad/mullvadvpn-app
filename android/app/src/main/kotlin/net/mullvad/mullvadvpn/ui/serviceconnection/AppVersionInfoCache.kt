package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.ui.VersionInfo

class AppVersionInfoCache(private val managementService: ManagementService) {
    fun versionInfo(): Flow<VersionInfo> =
        combine(
            managementService.versionInfo,
            managementService.settings.map { it.showBetaReleases }
        ) { appVersionInfo, showBetaReleases ->
            VersionInfo(
                currentVersion = BuildConfig.VERSION_NAME,
                upgradeVersion = appVersionInfo.suggestedUpgrade,
                isOutdated = appVersionInfo.suggestedUpgrade != null,
                isSupported = appVersionInfo.supported,
            )
        }
}
