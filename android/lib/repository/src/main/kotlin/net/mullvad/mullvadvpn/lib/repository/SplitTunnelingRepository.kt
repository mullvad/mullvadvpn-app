package net.mullvad.mullvadvpn.lib.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.SplitTunnelMode

class SplitTunnelingRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val splitTunnelingEnabled =
        managementService.settings
            .map { it.splitTunnelSettings.enabled }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), false)

    val excludedApps =
        managementService.settings
            .map { it.splitTunnelSettings.excludedApps }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), emptySet())

    val splitTunnelingMode =
        managementService.settings
            .map { it.splitTunnelSettings.mode }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), SplitTunnelMode.EXCLUDE)

    suspend fun enableSplitTunneling(enabled: Boolean) =
        managementService.setSplitTunnelingState(enabled)

    suspend fun excludeApp(app: AppId) = managementService.addSplitTunnelingApp(app)

    suspend fun includeApp(app: AppId) = managementService.removeSplitTunnelingApp(app)

    suspend fun setMode(mode: SplitTunnelMode) = managementService.setSplitTunnelingMode(mode)
}
