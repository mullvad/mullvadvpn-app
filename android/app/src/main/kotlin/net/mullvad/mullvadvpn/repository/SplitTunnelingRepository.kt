package net.mullvad.mullvadvpn.repository

import android.content.Context
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.util.migrateSplitTunneling
import net.mullvad.mullvadvpn.lib.common.util.removeOldSettings
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.AppId

class SplitTunnelingRepository(
    private val context: Context,
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val splitTunnelingEnabled =
        managementService.settings
            .map { it.splitTunnelSettings.enabled }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), false)

    val excludedApps =
        managementService.settings
            .map { it.splitTunnelSettings.excludedApps }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), emptySet())

    val migrateSplitTunnelingErrors = managementService.migrateSplitTunnelingError.receiveAsFlow()

    suspend fun enableSplitTunneling(enabled: Boolean) =
        managementService.setSplitTunnelingState(enabled)

    suspend fun excludeApp(app: String) = managementService.addSplitTunnelingApp(AppId(app))

    suspend fun includeApp(app: String) = managementService.removeSplitTunnelingApp(AppId(app))

    suspend fun tryMigrateSplitTunneling() =
        migrateSplitTunneling(
            context,
            { managementService.setSplitTunnelingState(it).isRight() },
            { appIds ->
                appIds
                    .map { appId -> managementService.addSplitTunnelingApp(appId).isRight() }
                    .all { it }
            },
        )

    fun clearOldSettings() {
        removeOldSettings(context)
    }

    suspend fun resetShouldTryMigrateSplitTunneling() {
        managementService.shouldTryMigrateSplitTunneling.emit(true)
    }
}
