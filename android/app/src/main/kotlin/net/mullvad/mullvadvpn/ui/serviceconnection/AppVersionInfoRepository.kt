package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.ui.VersionInfo

class AppVersionInfoRepository(
    private val buildVersion: BuildVersion,
    private val managementService: ManagementService
) {
    fun versionInfo(): Flow<VersionInfo> =
        managementService.versionInfo.map { appVersionInfo ->
            VersionInfo(currentVersion = buildVersion.name, isSupported = appVersionInfo.supported)
        }
}
