package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted.Companion.WhileSubscribed
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.shared.VersionInfo

class AppVersionInfoRepository(
    private val buildVersion: BuildVersion,
    managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val versionInfo: StateFlow<VersionInfo> =
        managementService.versionInfo
            .map { appVersionInfo ->
                VersionInfo(
                    currentVersion = buildVersion.name,
                    isSupported = appVersionInfo.supported,
                )
            }
            .stateIn(
                CoroutineScope(dispatcher),
                WhileSubscribed(),
                // By default we assume we are supported
                VersionInfo(currentVersion = buildVersion.name, isSupported = true),
            )
}
